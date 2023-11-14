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

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);
    let mut res = IndirectReservoir::default();
    let mut res_p_hat = 0.0;

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.gbuffer.depth == 0.0 {
        res.normalize(0.0);
        res.write(indirect_diffuse_spatial_reservoirs, screen_idx);
        return;
    }

    // -------------------------------------------------------------------------

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut sample = IndirectReservoir::read(
            prev_indirect_diffuse_spatial_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        sample.clamp_m(256.0);
        sample.m *= reprojection.confidence;

        let sample_p_hat =
            sample.sample.spatial_p_hat(hit.point, hit.gbuffer.normal);

        if res.merge(&mut wnoise, &sample, sample_p_hat) {
            res_p_hat = sample_p_hat;
        }
    }

    // ---

    let mut sample_fuel = if res.m < 250.0 { 6.0f32 } else { 3.0f32 };
    let mut sample_radius = 32.0f32;

    while sample_fuel > 0.0 {
        let sample_pos = screen_pos.as_vec2()
            + wnoise.sample_disk() * sample_radius.max(3.0);

        let sample_pos = camera.contain(sample_pos.as_ivec2());

        let sample_similarity =
            surface_map.get(sample_pos).evaluate_similarity_to(&surface);

        if sample_similarity < 0.5 {
            sample_fuel -= 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let mut sample = IndirectReservoir::read(
            indirect_diffuse_temporal_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            sample_fuel -= 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let sample_p_hat =
            sample.sample.spatial_p_hat(hit.point, hit.gbuffer.normal);

        if sample_p_hat < 0.0 {
            sample_fuel -= 0.5;
            sample_radius *= 0.75;
            continue;
        }

        let sample_jacobian = sample.sample.jacobian(hit.point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if sample_jacobian < 1.0 / 10.0 || sample_jacobian > 10.0 {
            sample_fuel -= 0.5;
            sample_radius *= 0.75;
            continue;
        }

        sample.m *= sample_similarity;

        if res.merge(&mut wnoise, &sample, sample_p_hat * sample_jacobian) {
            res_p_hat = sample_p_hat;
        }

        sample_fuel -= 1.0;
    }

    // -------------------------------------------------------------------------

    res.normalize(res_p_hat);
    res.write(indirect_diffuse_spatial_reservoirs, screen_idx);
}