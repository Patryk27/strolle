use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 4)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 5)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] prim_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);
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

    // ---

    let mut main = DiReservoir::default();
    let mut main_nth = 0;
    let mut main_pdf = 0.0;

    let mut curr_m = 0.0;
    let mut prev_m = 0.0;

    // ---

    let curr = DiReservoir::read(curr_reservoirs, screen_idx);

    if curr.m > 0.0 {
        let curr_pdf = curr.sample.pdf(lights, hit.point, hit.gbuffer.normal);

        if main.merge(&mut wnoise, &curr, curr_pdf) {
            main_nth = 1;
            main_pdf = curr_pdf;
        }

        curr_m = curr.m;
    }

    // ---

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        // let prev_pdf_w = BilinearFilter::reproject(reprojection, move |pos| {
        //     let res =
        //         DiReservoir::read(prev_reservoirs, camera.screen_to_idx(pos));

        //     if res.is_empty() {
        //         (Vec4::ZERO, 0.0)
        //     } else {
        //         let res_pdf =
        //             res.sample.pdf(lights, hit.point, hit.gbuffer.normal);

        //         if res_pdf > 0.0 {
        //             (vec4(res.w * res_pdf, 0.0, 0.0, 0.0), 1.0)
        //         } else {
        //             (Vec4::ZERO, 0.0)
        //         }
        //     }
        // })
        // .x;

        let mut prev = DiReservoir::read(
            prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        prev.clamp_m(20.0 * curr_m.max(1.0));

        let prev_pdf = if prev.sample.exists {
            prev.sample.pdf(lights, hit.point, hit.gbuffer.normal)
        } else {
            0.0
        };

        // if prev_pdf > 0.0 {
        //     prev.w = (prev_pdf_w / prev_pdf).clamp(0.0, 10.0);
        // }

        if main.merge(&mut wnoise, &prev, prev_pdf) {
            main_nth = 2;
            main_pdf = prev_pdf;
        }

        prev_m = prev.m;
    }

    // ---

    // let mut pi = main_pdf;
    // let mut pi_sum = main_pdf * curr_m;

    // if (prev_m > 0.0) & main.sample.exists {
    //     let ps = main.sample.pdf(lights, hit.point, hit.gbuffer.normal);

    //     pi = if main_nth == 2 { ps } else { pi };
    //     pi_sum += ps * prev_m;
    // }

    // main.normalize_ex(main_pdf, pi, pi_sum);

    main.normalize(main_pdf);
    main.write(curr_reservoirs, screen_idx);
}
