#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &DirectSpatialResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0)]
    blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)]
    prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    prev_direct_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        WhiteNoise::new(params.seed, global_id.xy()),
        BlueNoise::new(blue_noise_tex, global_id.xy(), params.frame),
        camera,
        SurfaceMap::new(surface_map),
        SurfaceMap::new(prev_surface_map),
        ReprojectionMap::new(reprojection_map),
        direct_temporal_reservoirs,
        direct_spatial_reservoirs,
        prev_direct_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    mut wnoise: WhiteNoise,
    bnoise: BlueNoise,
    camera: &Camera,
    surface_map: SurfaceMap,
    prev_surface_map: SurfaceMap,
    reprojection_map: ReprojectionMap,
    direct_temporal_reservoirs: &[Vec4],
    direct_spatial_reservoirs: &mut [Vec4],
    prev_direct_spatial_reservoirs: &[Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut reservoir = DirectReservoir::default();
    let screen_surface = surface_map.get(screen_pos);

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Try reprojecting reservoir from the previous frame; as an extra, we are
    // using bilinear filtering to reduce smearing in case of camera rotations
    // and movements.
    //
    // (Catmull-Rom would be even better but also much more expensive, so...)
    //
    // TODO if this fails (e.g. there's no reprojection due to occlusion), it
    //      would be nice to recover somehow -- maybe we could try merging
    //      nearby spatial reservoirs as well?

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut prev_reservoir = DirectReservoir::read(
            prev_direct_spatial_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        let default_sample = prev_reservoir
            .sample
            .light_contribution
            .extend(prev_reservoir.w);

        let filter =
            BilinearFilter::from_reprojection(reprojection, move |pos| {
                if !camera.contains(pos) {
                    return default_sample;
                }

                let pos = pos.as_uvec2();

                let reservoir = DirectReservoir::read(
                    prev_direct_spatial_reservoirs,
                    camera.screen_to_idx(pos),
                );

                if prev_surface_map
                    .get(pos)
                    .evaluate_similarity_to(&screen_surface)
                    < 0.33
                {
                    return default_sample;
                }

                if reservoir.sample.light_id == prev_reservoir.sample.light_id {
                    reservoir.sample.light_contribution.extend(reservoir.w)
                } else {
                    default_sample
                }
            });

        let sample = filter.eval_reprojection(reprojection);

        prev_reservoir.sample.light_contribution = sample.xyz();
        prev_reservoir.w = sample.w.clamp(0.0, 1000.0);
        prev_reservoir.m_sum *= reprojection.confidence;

        reservoir.merge(
            &mut wnoise,
            &prev_reservoir,
            prev_reservoir.sample.p_hat(),
        );
    }

    // -------------------------------------------------------------------------
    // Step 2:
    //
    // Analyze our screen-space neighbourhood and try to incorporate samples
    // from temporal reservoirs around us into our current reservoir.
    //
    // To get a good coverage, we follow a spiral pattern driven by blue noise:
    //
    // ```
    //         5
    //
    //
    //     1
    // 4   C 2
    //
    //     3
    //
    // (C - our current sample; 1..4 - order of looking at neighbours)
    // ```

    let mut p_hat = reservoir.sample.p_hat();
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.second_sample().x;
    let mut max_sample_radius = 12.0;

    while sample_radius < max_sample_radius {
        let rhs_pos = screen_pos.as_vec2()
            + vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

        let rhs_pos = rhs_pos.as_ivec2();

        sample_radius += 1.0;
        sample_angle += PI * 1.61803398875;

        if !camera.contains(rhs_pos) {
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();

        let rhs_similarity = surface_map.evaluate_similarity_between(
            screen_pos,
            screen_surface,
            rhs_pos,
        );

        if rhs_similarity < 0.5 {
            continue;
        }

        // If the surface we're looking at has very similar characteristics to
        // our center surface, there's a good chance we're looking at a wall or
        // a floor - i.e. a continuous, long surface.
        //
        // In this case, we'd like to extend the search radius to gather more
        // samples from far-away reservoirs to reduce boiling artifacts.
        if rhs_similarity > 0.75 {
            sample_radius += 1.0;
            max_sample_radius += 0.75;
        }

        // Since we don't perform occlusion checks, we can't randomly accept all
        // samples from far-away reservoirs, because that could attenuate small
        // shadows and make them unnaturally bright.
        //
        // At the same time, we *do* want to get some samples from those
        // far-away reservoirs to reduce boiling, so here's a middle ground:
        //
        // We accept samples from far-away reservoirs, but make them less
        // important.
        //
        // TODO implement a screen-space occlusion check
        let rhs_visibility = (1.0 - (sample_radius * 0.2).min(1.0)).max(0.33);

        let rhs = DirectReservoir::read(
            direct_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat();

        if reservoir.merge(
            &mut wnoise,
            &rhs,
            rhs_p_hat * rhs_similarity * rhs_visibility,
        ) {
            p_hat = rhs_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 1000.0, 500.0);
    reservoir.write(direct_spatial_reservoirs, screen_idx);
}
