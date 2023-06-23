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
    past_direct_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        SurfaceMap::new(surface_map),
        ReprojectionMap::new(reprojection_map),
        direct_temporal_reservoirs,
        direct_spatial_reservoirs,
        past_direct_spatial_reservoirs,
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
    past_direct_spatial_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let global_idx = camera.screen_to_idx(screen_pos);
    let mut reservoir = DirectReservoir::default();

    // -------------------------------------------------------------------------

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_valid() {
        let mut past_reservoir = DirectReservoir::read(
            past_direct_spatial_reservoirs,
            camera.screen_to_idx(reprojection.past_screen_pos()),
        );

        past_reservoir.m_sum *=
            (reprojection.confidence * reprojection.confidence).max(0.1);

        reservoir.merge(
            &mut noise,
            &past_reservoir,
            past_reservoir.sample.p_hat(),
        );
    }

    // -------------------------------------------------------------------------

    let mut p_hat = reservoir.sample.p_hat();
    let screen_surface = surface_map.get(screen_pos);

    let mut sample_idx = 0;
    let mut sample_radius = 24.0f32;

    while sample_idx < 4 {
        let rhs_pos =
            screen_pos.as_vec2() + noise.sample_disk() * sample_radius.max(3.0);

        let rhs_pos = rhs_pos.as_ivec2();

        if !camera.contains(rhs_pos) {
            sample_idx += 1;
            sample_radius *= 0.5;
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();
        let rhs_surface = surface_map.get(rhs_pos);
        let rhs_similarity = screen_surface.evaluate_similarity_to(rhs_surface);

        if rhs_similarity < 0.5 {
            sample_idx += 1;
            sample_radius *= 0.5;
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

        sample_idx += 1;
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 500.0);
    reservoir.write(direct_spatial_reservoirs, global_idx);
}
