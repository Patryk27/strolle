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
    stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0)]
    blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 5, uniform)]
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
    main_inner(
        global_id.xy(),
        local_idx,
        BlueNoise::new(blue_noise_tex,  global_id.xy(), params.frame),
        WhiteNoise::new(params.seed, global_id.xy()),
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
    bnoise: BlueNoise,
    mut wnoise: WhiteNoise,
    stack: BvhStack,
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
    // Step 1:
    //
    // Select the best light-candidate, judging lights by their *unshadowed*
    // contribution (i.e. during this phase we don't cast shadow rays).

    let mut reservoir = DirectReservoir::default();

    if hit.is_some() {
        let material = materials.get(hit.material_id);
        let mut light_idx = 0;

        while light_idx < world.light_count {
            let light_id = LightId::new(light_idx);

            let light_contribution = lights
                .get(light_id)
                .contribution(material, hit, ray)
                .diffuse;

            let sample = DirectReservoirSample {
                light_id,
                light_contribution,
            };

            // TODO shouldn't we incorporate light's PDF as well?
            reservoir.add(&mut wnoise, sample, sample.p_hat());
            light_idx += 1;
        }
    } else {
        // If we hit nothing, the reservoir will remain empty and we'll just
        // sample the sky in a moment.
    }

    // If the reservoir is empty (i.e. it has seen zero samples or all of the
    // samples are unusable¹), sample the sky.
    //
    // If the reservoir has seen some samples, sample the sky with 25%
    // probability (following the ReSTIR paper).
    //
    // ¹ e.g. are lights very very far away from here
    let sky_weight = if reservoir.w_sum == 0.0 {
        1.0
    } else {
        0.25 * reservoir.w_sum
    };

    if sky_weight > 0.0 {
        // If we hit nothing, sample the sky - otherwise, sample the sun, so:
        //
        // - if the user is looking at the sky, we get no-hit and sample the
        //   sky,
        //
        // - if the user is looking at the world, we get hit and sample the sun
        //   so that the sun is able to cast shadows.
        let sky = if hit.is_none() {
            atmosphere.eval(world.sun_direction(), ray.direction())
        } else {
            atmosphere.sun(world.sun_direction())
        };

        let sample = DirectReservoirSample::sky(sky);

        reservoir.add(&mut wnoise, sample, sky_weight * sample.p_hat());
    }

    // -------------------------------------------------------------------------
    // Step 2:
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
        let light = if reservoir.sample.is_sky() {
            Light::sun(world.sun_position())
        } else {
            lights.get(reservoir.sample.light_id)
        };

        light.visibility_bnoise(local_idx, triangles, bvh, stack, bnoise, hit)
    };

    let light = light_contribution * light_visibility;

    unsafe {
        *direct_initial_samples.get_unchecked_mut(screen_idx) =
            light.extend(f32::from_bits(light_id.get()));
    }
}
