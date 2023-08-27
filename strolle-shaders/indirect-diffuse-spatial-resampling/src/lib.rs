#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_diffuse_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_diffuse_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)]
    prev_indirect_diffuse_spatial_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let surface_map = SurfaceMap::new(surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);
    let mut reservoir = IndirectReservoir::default();

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.gbuffer.depth == 0.0 {
        reservoir.normalize(0.0, 10.0, 500.0);
        reservoir.write(indirect_diffuse_spatial_reservoirs, screen_idx);
        return;
    }

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Try reprojecting reservoir from the previous frame.

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut rhs = IndirectReservoir::read(
            prev_indirect_diffuse_spatial_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        rhs.m_sum *= reprojection.confidence;

        reservoir.merge(
            &mut wnoise,
            &rhs,
            rhs.sample.spatial_p_hat(hit.point, hit.gbuffer.normal),
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

    let mut p_hat = reservoir
        .sample
        .spatial_p_hat(hit.point, hit.gbuffer.normal);

    let mut sample_idx = 0.0f32;
    let mut sample_radius = 32.0f32;

    while sample_idx <= 6.0 {
        let rhs_pos = screen_pos.as_vec2()
            + wnoise.sample_disk() * sample_radius.max(3.0);

        let rhs_pos = camera.contain(rhs_pos.as_ivec2());

        let rhs_similarity = surface_map
            .evaluate_similarity_between(screen_pos, surface, rhs_pos);

        if rhs_similarity < 0.5 {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs = IndirectReservoir::read(
            indirect_diffuse_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        if rhs.is_empty() {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs_p_hat = rhs.sample.spatial_p_hat(hit.point, hit.gbuffer.normal);

        if rhs_p_hat < 0.0 {
            sample_idx += 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let rhs_jacobian = rhs.sample.jacobian(hit.point);

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
    reservoir.write(indirect_diffuse_spatial_reservoirs, screen_idx);
}
