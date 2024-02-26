use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &GiPreviewResamplingPass,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    in_reservoirs_a: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    in_reservoirs_b: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    out_reservoirs: &mut [Vec4],
) {
    let center_pos = global_id.xy();
    let center_idx = camera.screen_to_idx(center_pos);
    let mut wnoise = WhiteNoise::new(params.seed, center_pos);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(center_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let center_hit = Hit::new(
        camera.ray(center_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(center_pos),
            prim_gbuffer_d1.read(center_pos),
        ]),
    );

    if center_hit.is_none() {
        GiReservoir::default().write(out_reservoirs, center_idx);
        return;
    }

    // -------------------------------------------------------------------------

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    // ---

    let center = if params.source == 0 {
        GiReservoir::read(in_reservoirs_a, center_idx)
    } else {
        GiReservoir::read(in_reservoirs_b, center_idx)
    };

    if main.merge(&mut wnoise, &center, center.sample.pdf) {
        main_pdf = center.sample.pdf;
    }

    // ---

    let max_samples;
    let max_radius;

    if params.nth == 0 {
        max_samples = lerp(8.0, 0.0, main.m / 8.0) as u32;
        max_radius = 128.0;
    } else {
        max_samples = lerp(8.0, 0.0, main.m / 8.0) as u32;
        max_radius = 64.0;
    }

    // ---

    let mut sample_nth = 0;

    while sample_nth < max_samples {
        sample_nth += 1;

        let sample_pos = camera.contain(
            (center_pos.as_vec2() + wnoise.sample_disk() * max_radius)
                .as_ivec2(),
        );

        if sample_pos == center_pos {
            return;
        }

        let sample_surface = prim_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        if (sample_surface.depth - center_hit.gbuffer.depth).abs()
            > 0.25 * center_hit.gbuffer.depth
        {
            continue;
        }

        if sample_surface.normal.dot(center_hit.gbuffer.normal) < 0.5 {
            continue;
        }

        let sample = if params.source == 0 {
            GiReservoir::read(in_reservoirs_a, camera.screen_to_idx(sample_pos))
        } else {
            GiReservoir::read(in_reservoirs_b, camera.screen_to_idx(sample_pos))
        };

        if sample.is_empty() {
            continue;
        }

        let sample_pdf = sample.sample.pdf(center_hit);
        let sample_jacobian = sample.sample.jacobian(center_hit.point);

        // TODO rust-gpu seems to miscompile `.contains()`
        #[allow(clippy::manual_range_contains)]
        if sample_jacobian < 1.0 / 10.0 || sample_jacobian > 10.0 {
            continue;
        }

        let sample_jacobian = sample_jacobian.clamp(1.0 / 3.0, 3.0);

        if main.merge(&mut wnoise, &sample, sample_pdf * sample_jacobian) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    main.confidence = center.confidence;
    main.sample.pdf = main_pdf;
    main.sample.v1_point = center.sample.v1_point;
    main.norm_avg(main_pdf);
    main.clamp_w(5.0);
    main.write(out_reservoirs, center_idx);
}
