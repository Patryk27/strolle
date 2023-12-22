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
    let mut main_pdf = 0.0;

    // ---

    let sample =
        IndirectReservoir::read(indirect_diffuse_input_reservoirs, screen_idx);

    let sample_pdf = sample.sample.spatial_pdf(&hit);

    if main.merge(&mut wnoise, &sample, sample_pdf) {
        main_pdf = sample_pdf;
    }

    // ---

    let max_samples;
    let max_radius;

    if params.nth == 1 {
        max_samples = lerp(8.0, 2.0, main.m / 64.0) as u32;
        max_radius = 32.0;
    } else {
        max_samples = lerp(4.0, 0.0, main.m / 128.0) as u32;
        max_radius = 16.0;
    }

    // ---

    let mut sample_idx = 0;

    while sample_idx < max_samples {
        let sample_pos = {
            let sample_pos =
                screen_pos.as_vec2() + wnoise.sample_disk() * max_radius;

            camera.contain(sample_pos.as_ivec2())
        };

        sample_idx += 1;

        let sample_surface = surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        if (sample_surface.depth - surface.depth).abs() > 0.2 * surface.depth {
            continue;
        }

        if sample_surface.normal.dot(surface.normal) < 0.8 {
            continue;
        }

        let sample = IndirectReservoir::read(
            indirect_diffuse_input_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            continue;
        }

        let sample_pdf = sample.sample.spatial_pdf(&hit);

        if sample_pdf < 0.0 {
            continue;
        }

        let sample_jacobian = sample.sample.jacobian(hit.point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if sample_jacobian < 1.0 / 10.0 || sample_jacobian > 10.0 {
            continue;
        }

        let sample_jacobian = sample_jacobian.clamp(1.0 / 3.0, 3.0).sqrt();

        if main.merge(&mut wnoise, &sample, sample_pdf * sample_jacobian) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_pdf);
    main.write(indirect_diffuse_output_reservoirs, screen_idx);
}
