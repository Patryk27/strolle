use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    prev_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, global_id.xy());
    let prim_surface_map = SurfaceMap::new(prim_surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let surface = prim_surface_map.get(screen_pos);
    let reprojection = reprojection_map.get(screen_pos);

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    // ---

    let hit_point = camera.ray(screen_pos).at(surface.depth);
    let hit_normal = surface.normal;

    if is_checkerboard(screen_pos, params.frame) {
        let d0 = unsafe { *samples.index_unchecked(3 * screen_idx) };
        let d1 = unsafe { *samples.index_unchecked(3 * screen_idx + 1) };
        let d2 = unsafe { *samples.index_unchecked(3 * screen_idx + 2) };

        if d0.w.to_bits() == 1 {
            let sample = GiSample {
                radiance: d1.xyz(),
                v1_point: d0.xyz(),
                v2_point: d2.xyz(),
                v2_normal: Normal::decode(vec2(d1.w, d2.w)),
                frame: params.frame,
            };

            main_pdf = sample.diff_pdf(hit_point, hit_normal);
            main.update(&mut wnoise, sample, main_pdf);
        }
    }

    // ---

    if reprojection.is_some() {
        let prev_pdf_w = BilinearFilter::reproject(reprojection, move |pos| {
            let res =
                GiReservoir::read(prev_reservoirs, camera.screen_to_idx(pos));

            if res.is_empty() {
                (Vec4::ZERO, 0.0)
            } else {
                let res_pdf = res.sample.diff_pdf(hit_point, hit_normal);

                if res_pdf > 0.0 {
                    (vec4(res.w * res_pdf, 0.0, 0.0, 0.0), 1.0)
                } else {
                    (Vec4::ZERO, 0.0)
                }
            }
        })
        .x;

        let mut prev = GiReservoir::read(
            prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        prev.clamp_m(64.0);

        if !prev.is_empty() {
            let prev_pdf = prev.sample.diff_pdf(hit_point, hit_normal);

            if prev_pdf > 0.0 {
                prev.w = (prev_pdf_w / prev_pdf).clamp(0.0, 10.0);
            }

            if main.merge(&mut wnoise, &prev, prev_pdf) {
                main_pdf = prev_pdf;
            }
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_pdf);
    main.write(curr_reservoirs, screen_idx);
}
