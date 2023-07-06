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
    params: &IndirectInitialShadingPassParams,
    #[spirv(workgroup)]
    stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4)]
    atlas_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 5)]
    atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 6, uniform)]
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
    #[spirv(descriptor_set = 1, binding = 7)]
    indirect_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 8)]
    indirect_hits_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 9, storage_buffer)]
    indirect_initial_samples: &mut [Vec4],
) {
    main_inner(
        global_id.xy(),
        local_idx,
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
        atlas_tex,
        atlas_sampler,
        world,
        camera,
        direct_hits_d0,
        direct_hits_d1,
        indirect_hits_d0,
        indirect_hits_d1,
        indirect_initial_samples,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    local_idx: u32,
    mut wnoise: WhiteNoise,
    stack: BvhStack,
    triangles: TrianglesView,
    bvh: BvhView,
    lights: LightsView,
    materials: MaterialsView,
    atmosphere: Atmosphere,
    atlas_tex: &Image!(2D, type=f32, sampled),
    atlas_sampler: &Sampler,
    world: &World,
    camera: &Camera,
    direct_hits_d0: TexRgba32f,
    direct_hits_d1: TexRgba32f,
    indirect_hits_d0: TexRgba32f,
    indirect_hits_d1: TexRgba32f,
    indirect_initial_samples: &mut [Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);

    // -------------------------------------------------------------------------

    let direct_hit = Hit::deserialize(
        direct_hits_d0.read(screen_pos),
        direct_hits_d1.read(screen_pos),
    );

    if direct_hit.is_none() {
        unsafe {
            *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 0) =
                Default::default();

            *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 1) =
                Default::default();

            *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 2) =
                Default::default();
        }

        return;
    }

    let indirect_ray = Ray::new(
        direct_hit.point,
        wnoise.sample_hemisphere(direct_hit.normal),
    );

    let indirect_hit = Hit::deserialize(
        indirect_hits_d0.read(screen_pos),
        indirect_hits_d1.read(screen_pos),
    );

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Similarly as for direct lightning, let's start by selecting the best
    // light-candidate, judging lights by their *unshadowed* contribution.
    //
    // This algorithm follows a similar logic as direct initial shading, so
    // comments were skipped for brevity.

    let mut reservoir = DirectReservoir::default();

    if indirect_hit.is_some() {
        let mut material = materials.get(indirect_hit.material_id);

        material.adjust_for_indirect();

        let albedo = material
            .albedo(atlas_tex, atlas_sampler, indirect_hit.uv)
            .xyz();

        let mut light_idx = 0;

        while light_idx < world.light_count {
            let light_id = LightId::new(light_idx);

            let light_contribution = lights
                .get(light_id)
                .contribution(material, indirect_hit, indirect_ray)
                .with_albedo(albedo)
                .sum();

            let sample = DirectReservoirSample {
                light_id,
                light_contribution,
            };

            // TODO shouldn't we incorporate light's PDF as well?
            reservoir.add(&mut wnoise, sample, sample.p_hat());
            light_idx += 1;
        }
    }

    let sky_weight = if reservoir.w_sum == 0.0 {
        1.0
    } else {
        0.25 * reservoir.w_sum
    };

    let mut sky_normal = Vec3::ZERO;

    if sky_weight > 0.0 {
        // If we indirectly-hit nothing, we know that our indirect-ray must be
        // pointing towards the sky - great, let's use it!
        //
        // If we indirectly-hit something, we don't know in which way we can
        // sample the sky, so just take a random guess on the hemisphere on our
        // surface.
        sky_normal = if indirect_hit.is_none() {
            indirect_ray.direction()
        } else {
            wnoise.sample_hemisphere(indirect_hit.normal)
        };

        // Cursed:
        //
        // Since we only support single-bounce GI, let's arbitrarily boost the
        // sky's exposure to compensate for the missing bounces.
        //
        // It's pretty so-so (and increases variance), but it helps a bit as
        // well.
        let sky_exposure = if indirect_hit.is_some() { 9.0 } else { 4.5 };

        let sample = DirectReservoirSample::sky(
            sky_exposure * atmosphere.eval(world.sun_direction(), sky_normal),
        );

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

    let light = if reservoir.sample.is_sky() {
        Light::sun(sky_normal * World::SUN_DISTANCE)
    } else {
        lights.get(light_id)
    };

    // If we indirectly-hit nothing, we know that our indirect-ray must be
    // pointing towards the sky - great, no need to actually trace the ray!
    let light_visibility = if indirect_hit.is_none() {
        1.0
    } else {
        light.visibility(
            local_idx,
            triangles,
            bvh,
            stack,
            &mut wnoise,
            indirect_hit,
        )
    };

    let color = light_contribution
        * light_visibility
        * indirect_ray.direction().dot(direct_hit.normal);

    let indirect_normal;
    let indirect_point;

    if indirect_hit.is_some() {
        indirect_normal = Normal::encode(indirect_hit.normal);
        indirect_point = indirect_hit.point;
    } else {
        indirect_normal = Normal::encode(-sky_normal);
        indirect_point = sky_normal * World::SUN_DISTANCE;
    }

    unsafe {
        *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 0) =
            color.extend(indirect_normal.x);

        *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 1) =
            direct_hit.point.extend(indirect_normal.y);

        *indirect_initial_samples.get_unchecked_mut(3 * screen_idx + 2) =
            indirect_point.extend(f32::from_bits(1));
    }
}
