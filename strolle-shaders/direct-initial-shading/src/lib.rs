#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(local_invocation_index)]
    local_idx: u32,
    #[spirv(push_constant)]
    params: &DirectInitialShadingPassParams,
    #[spirv(workgroup)]
    stack: BvhTraversingStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[BvhNode],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4, uniform)]
    world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    atmosphere_transmittance_lut_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 2)]
    atmosphere_transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 3)]
    atmosphere_sky_lut_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 1, binding = 4)]
    atmosphere_sky_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 5)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)]
    direct_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    direct_initial_samples: &mut [Vec4],
) {
    let noise = Noise::new(params.seed, global_id.xy());

    main_inner(
        global_id.xy(),
        local_idx,
        noise,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        LightsView::new(lights),
        MaterialsView::new(materials),
        Atmosphere::new(
            atmosphere_transmittance_lut_tex,
            atmosphere_transmittance_lut_sampler,
            atmosphere_sky_lut_tex,
            atmosphere_sky_lut_sampler,
        ),
        world,
        camera,
        direct_hits_d0,
        direct_hits_d1,
        direct_initial_samples,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    local_idx: u32,
    mut noise: Noise,
    stack: BvhTraversingStack,
    triangles: TrianglesView,
    bvh: BvhView,
    lights: LightsView,
    materials: MaterialsView,
    atmosphere: Atmosphere,
    world: &World,
    camera: &Camera,
    direct_hits_d0: TexRgba32f,
    direct_hits_d1: TexRgba32f,
    direct_initial_samples: &mut [Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);
    let ray = camera.ray(screen_pos);

    let hit = Hit::deserialize(
        direct_hits_d0.read(screen_pos),
        direct_hits_d1.read(screen_pos),
    );

    // -------------------------------------------------------------------------
    // Phase 1:
    //
    // Select the best light-candidate, judging lights by their *unshadowed*
    // contribution (i.e. during this phase we don't cast shadow rays).

    let mut reservoir = DirectReservoir::default();

    if hit.is_some() {
        let material = materials.get(MaterialId::new(hit.material_id));
        let mut light_id = 0;

        while light_id < world.light_count {
            // We're hard-coding albedo here because later, during the resolving
            // pass, we're going to re-compute this light's contribution anyway,
            // this time using the correct albedo.
            //
            // We do that because if the object we're shading is textured, we
            // would carry nasty reprojection artifacts through from reservoirs up
            // to the user, and those look *bad*.
            //
            // Currently those reprojection artifacts simply stay in temporal and
            // spatial reservoirs, and just cause some bias - that's managable, tho.
            let albedo = Vec3::ONE;

            // TODO add support for specular lightning
            let light_contribution = lights
                .get(LightId::new(light_id))
                .contribution(material, hit, ray, albedo)
                .diffuse;

            let sample = DirectReservoirSample {
                light_id,
                light_contribution,
            };

            reservoir.add(&mut noise, sample, sample.p_hat());
            light_id += 1;
        }
    }

    let sky_weight = if reservoir.w_sum == 0.0 {
        1.0
    } else {
        0.25 * reservoir.w_sum
    };

    if sky_weight > 0.0 {
        let sky = if hit.is_none() {
            atmosphere.eval(world.sun_direction(), ray.direction())
        } else {
            atmosphere.sun(world.sun_direction())
        };

        reservoir.add(
            &mut noise,
            DirectReservoirSample {
                light_id: u32::MAX,
                light_contribution: sky,
            },
            sky_weight,
        );
    }

    // -------------------------------------------------------------------------
    // Phase 2:
    //
    // Select the best light-candidate and cast a shadow ray to check if that
    // light (which might be sun) is actually visible to us.

    let DirectReservoirSample {
        light_id,
        light_contribution,
    } = reservoir.sample;

    let light_visibility = if hit.is_none() {
        1.0
    } else {
        let light = if reservoir.sample.light_id == u32::MAX {
            Light::sun(world.sun_position())
        } else {
            lights.get(LightId::new(reservoir.sample.light_id))
        };

        light.visibility(local_idx, triangles, bvh, stack, &mut noise, hit)
    };

    let light = light_contribution * light_visibility;

    // Setting a mininimum radiance is technically wrong but at least this
    // way we don't have to deal with zero p_hats:
    let light = light.max(Vec3::splat(0.000001));

    direct_initial_samples[screen_idx] = light.extend(f32::from_bits(light_id));
}
