use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &GiDiffSpatialResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    input_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    output_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = prim_surface_map.get(screen_pos);

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.gbuffer.depth == 0.0 {
        GiReservoir::default().write(output_reservoirs, screen_idx);

        return;
    }

    // -------------------------------------------------------------------------

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    // ---

    let sample = GiReservoir::read(input_reservoirs, screen_idx);
    let sample_pdf = sample.sample.diffuse_pdf(hit.point, hit.gbuffer.normal);

    if main.merge(&mut wnoise, &sample, sample_pdf) {
        main_pdf = sample_pdf;
    }

    let main_luma = sample.sample.radiance.luminance()
        * sample.w
        * sample.sample.cosine(&hit);

    let max_luma_diff = lerp(4.0, 0.25, main.m / 20.0);

    // ---

    let max_samples;
    let max_radius;

    if params.nth == 1 {
        max_samples = lerp(8.0, 3.0, main.m / 50.0) as u32;
        max_radius = 64.0;
    } else {
        max_samples = lerp(4.0, 1.0, main.m / 100.0) as u32;
        max_radius = 32.0;
    }

    // ---

    let mut sample_idx = 0;

    while sample_idx < max_samples {
        let sample_dist = wnoise.sample_disk() * max_radius;

        let sample_pos =
            camera.contain((screen_pos.as_vec2() + sample_dist).as_ivec2());

        sample_idx += 1;

        let sample_surface = prim_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        if (sample_surface.depth - surface.depth).abs() > 0.2 * surface.depth {
            continue;
        }

        if sample_surface.normal.dot(surface.normal) < 0.8 {
            continue;
        }

        let mut sample = GiReservoir::read(
            input_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            continue;
        }

        sample.clamp_m(lerp(64.0, 8.0, sample_dist.length() / max_radius));

        let sample_pdf =
            sample.sample.diffuse_pdf(hit.point, hit.gbuffer.normal);

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

        let sample_luma = sample.sample.radiance.luminance()
            * sample.w
            * sample.sample.cosine(&hit);

        if (sample_luma - main_luma).abs().sqrt() >= max_luma_diff {
            continue;
        }

        if main.merge(&mut wnoise, &sample, sample_pdf * sample_jacobian) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_pdf);
    main.write(output_reservoirs, screen_idx);
}
