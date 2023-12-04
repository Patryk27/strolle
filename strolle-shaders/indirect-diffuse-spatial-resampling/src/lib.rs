#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &IndirectDiffuseSpatialResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4)] _reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_diffuse_input_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_diffuse_output_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let surface_map = SurfaceMap::new(surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.gbuffer.depth == 0.0 {
        IndirectReservoir::default()
            .write(indirect_diffuse_output_reservoirs, screen_idx);

        return;
    }

    // -------------------------------------------------------------------------

    let mut main = IndirectReservoir::default();
    let mut main_p_hat = 0.0;

    // ---

    let sample =
        IndirectReservoir::read(indirect_diffuse_input_reservoirs, screen_idx);

    if !sample.is_empty() {
        let sample_p_hat = sample.sample.spatial_p_hat(&hit);

        if sample_p_hat >= 0.0 && main.merge(&mut wnoise, &sample, sample_p_hat)
        {
            main_p_hat = sample_p_hat;
        }
    }

    // ---

    let max_samples;
    let max_radius;

    if params.nth == 1 {
        max_samples = 4;
        max_radius = 32.0;
    } else {
        max_samples = 4;
        max_radius = 8.0;
    }

    // ---

    let mut sample_idx = 0;
    let mut sample_angle = wnoise.sample() * 2.0 * PI;

    while sample_idx < max_samples {
        let sample_offset = {
            let angle = vec2(sample_angle.cos(), sample_angle.sin());

            let radius = lerp(
                2.0,
                max_radius,
                (sample_idx as f32) / (max_samples as f32),
            );

            angle * radius
        };

        let sample_pos = screen_pos.as_vec2() + sample_offset;
        let sample_pos = camera.contain(sample_pos.as_ivec2());

        sample_idx += 1;
        sample_angle += GOLDEN_ANGLE;

        let sample_similarity =
            surface_map.get(sample_pos).evaluate_similarity_to(&surface);

        if sample_similarity < 0.5 {
            continue;
        }

        let mut sample = IndirectReservoir::read(
            indirect_diffuse_input_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            continue;
        }

        let sample_p_hat = sample.sample.spatial_p_hat(&hit);

        if sample_p_hat < 0.0 {
            continue;
        }

        let sample_jacobian = sample.sample.jacobian(hit.point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if sample_jacobian < 1.0 / 10.0 || sample_jacobian > 10.0 {
            continue;
        }

        let sample_jacobian = sample_jacobian.clamp(1.0 / 3.0, 3.0);

        sample.m *= sample_similarity;

        if main.merge(&mut wnoise, &sample, sample_p_hat * sample_jacobian) {
            main_p_hat = sample_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_p_hat);
    main.write(indirect_diffuse_output_reservoirs, screen_idx);
}
