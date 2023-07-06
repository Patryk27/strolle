#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &IndirectSpatialResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    direct_hits_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)]
    surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    prev_indirect_spatial_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        WhiteNoise::new(params.seed, global_id.xy()),
        camera,
        direct_hits_d0,
        SurfaceMap::new(surface_map),
        ReprojectionMap::new(reprojection_map),
        indirect_temporal_reservoirs,
        indirect_spatial_reservoirs,
        prev_indirect_spatial_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    mut wnoise: WhiteNoise,
    camera: &Camera,
    direct_hits_d0: TexRgba32f,
    surface_map: SurfaceMap,
    reprojection_map: ReprojectionMap,
    indirect_temporal_reservoirs: &[Vec4],
    indirect_spatial_reservoirs: &mut [Vec4],
    prev_indirect_spatial_reservoirs: &[Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut reservoir = IndirectReservoir::default();
    let screen_surface = surface_map.get(screen_pos);

    let direct_hit_point =
        Hit::deserialize_point(direct_hits_d0.read(screen_pos));

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Try reprojecting reservoir from the previous frame.
    //
    // TODO we could use some interpolation here, but it's kinda difficult to
    //      apply due to extra metadata stored in the reservoirs (sample normals
    //      etc.)

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut prev_reservoir = IndirectReservoir::read(
            prev_indirect_spatial_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

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
    // As compared to the direct-spatial-resampling pass, in here we simply
    // gather a few random samples around our current pixel and call it a day.
    //
    // TODO consider using blue noise (initial experiments proven it to be
    //      somewhat less stable here, though)

    let mut p_hat = reservoir.sample.p_hat();
    let mut sample_idx = 0.0f32;
    let mut sample_radius = 32.0f32;

    let max_samples = if reservoir.m_sum <= 350.0 { 6.0 } else { 3.0 };

    while sample_idx <= max_samples {
        let rhs_pos = screen_pos.as_vec2()
            + wnoise.sample_disk() * sample_radius.max(3.0);

        let rhs_pos = rhs_pos.as_ivec2();

        if !camera.contains(rhs_pos) {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();

        let rhs_similarity = surface_map.evaluate_similarity_between(
            screen_pos,
            screen_surface,
            rhs_pos,
        );

        if rhs_similarity < 0.5 {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs = IndirectReservoir::read(
            indirect_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat();
        let rhs_jacobian = rhs.sample.jacobian(direct_hit_point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if rhs_jacobian < 1.0 / 10.0 || rhs_jacobian > 10.0 {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs_jacobian = rhs_jacobian.clamp(1.0 / 3.0, 3.0);

        if reservoir.merge(
            &mut wnoise,
            &rhs,
            rhs_p_hat * rhs_similarity * rhs_jacobian,
        ) {
            p_hat = rhs_p_hat;
        }

        sample_idx += 1.0;
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 500.0);
    reservoir.write(indirect_spatial_reservoirs, screen_idx);
}
