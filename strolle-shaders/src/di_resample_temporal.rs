use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    rt_hits: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let lights = LightsView::new(lights);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            prim_gbuffer_d0.read(screen_pos),
            prim_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.is_none() {
        return;
    }

    let mut main = DiReservoir::default();
    let mut main_pdf = 0.0;

    // -------------------------------------------------------------------------

    let mut sample = DiReservoir::read(curr_reservoirs, screen_idx);

    sample.sample.is_occluded =
        unsafe { rt_hits.index_unchecked(2 * screen_idx).x.to_bits() == 1 };

    if sample.sample.is_occluded {
        sample.w = 0.0;
    }

    if sample.m > 0.0 {
        let sample_pdf =
            sample.sample.pdf(lights, hit.point, hit.gbuffer.normal);

        if main.merge(&mut wnoise, &sample, sample_pdf) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let sample_pdf_w =
            BilinearFilter::reproject(reprojection, move |pos| {
                let res = DiReservoir::read(
                    prev_reservoirs,
                    camera.screen_to_idx(pos),
                );

                if res.is_empty() {
                    (Vec4::ZERO, 0.0)
                } else {
                    let res_pdf =
                        res.sample.pdf(lights, hit.point, hit.gbuffer.normal);

                    if res_pdf > 0.0 {
                        (vec4(res.w * res_pdf, 0.0, 0.0, 0.0), 1.0)
                    } else {
                        (Vec4::ZERO, 0.0)
                    }
                }
            })
            .x;

        let mut sample = DiReservoir::read(
            prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        sample.clamp_m(20.0 * main.m.max(1.0));

        let sample_pdf = if sample.sample.exists {
            sample.sample.pdf(lights, hit.point, hit.gbuffer.normal)
        } else {
            0.0
        };

        // TODO
        if sample_pdf > 0.0 && false {
            sample.w = (sample_pdf_w / sample_pdf).clamp(0.0, 10.0);
        }

        if main.merge(&mut wnoise, &sample, sample_pdf) {
            main_pdf = sample_pdf;
        }
    }

    // ---

    main.normalize(main_pdf);
    main.write(curr_reservoirs, screen_idx);
}
