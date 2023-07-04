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
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    direct_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    direct_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    prev_direct_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        SurfaceMap::new(surface_map),
        ReprojectionMap::new(reprojection_map),
        direct_temporal_reservoirs,
        direct_spatial_reservoirs,
        prev_direct_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    params: &DirectSpatialResamplingPassParams,
    camera: &Camera,
    surface_map: SurfaceMap,
    reprojection_map: ReprojectionMap,
    direct_temporal_reservoirs: &[Vec4],
    direct_spatial_reservoirs: &mut [Vec4],
    prev_direct_spatial_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut reservoir = DirectReservoir::default();

    // -------------------------------------------------------------------------

    let screen_surface = surface_map.get(screen_pos);
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

                if reservoir.sample.light_id != prev_reservoir.sample.light_id {
                    return default_sample;
                }

                if surface_map.get(pos).evaluate_similarity_to(&screen_surface)
                    < 0.33
                {
                    return default_sample;
                }

                reservoir.sample.light_contribution.extend(reservoir.w)
            });

        let sample = filter.eval_reprojection(reprojection);

        prev_reservoir.sample.light_contribution = sample.xyz();
        prev_reservoir.w = sample.w.clamp(0.0, 1000.0);
        prev_reservoir.m_sum *= reprojection.confidence;

        reservoir.merge(
            &mut noise,
            &prev_reservoir,
            prev_reservoir.sample.p_hat(),
        );
    }

    // -------------------------------------------------------------------------

    let mut p_hat = reservoir.sample.p_hat();

    let mut sample_idx = 0.0f32;
    let mut sample_radius = 12.0f32;

    while sample_idx <= 4.0 {
        let rhs_pos =
            screen_pos.as_vec2() + noise.sample_disk() * sample_radius.max(3.0);

        let rhs_pos = rhs_pos.as_ivec2();

        if !camera.contains(rhs_pos) {
            sample_idx += 0.25;
            sample_radius *= 0.75;
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();

        // TODO add a screen-space occlusion check
        let rhs_similarity = surface_map.evaluate_similarity_between(
            screen_pos,
            screen_surface,
            rhs_pos,
        );

        if rhs_similarity < 0.5 {
            sample_idx += 0.25;
            sample_radius *= 0.75;
            continue;
        }

        let rhs = DirectReservoir::read(
            direct_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat();

        if reservoir.merge(&mut noise, &rhs, rhs_p_hat * rhs_similarity) {
            p_hat = rhs_p_hat;
        }

        sample_idx += 1.0;
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 1000.0, 125.0);
    reservoir.write(direct_spatial_reservoirs, screen_idx);
}
