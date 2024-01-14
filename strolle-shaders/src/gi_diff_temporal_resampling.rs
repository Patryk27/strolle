use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prev_prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    curr_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    prev_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, global_id.xy());
    let prim_surface_map = SurfaceMap::new(prim_surface_map);
    let prev_prim_surface_map = SurfaceMap::new(prev_prim_surface_map);
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
        let mut sample = GiReservoir::read(
            prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        sample.clamp_m(50.0);

        if !sample.is_empty() {
            let sample_pdf = sample.sample.diff_pdf(hit_point, hit_normal);

            if main.merge(&mut wnoise, &sample, sample_pdf) {
                main_pdf = sample_pdf;
            }
        }
    }

    // ---

    let mut sample_idx = 0;

    while main.m < 16.0 && sample_idx < 4 {
        let mut sample_pos = if reprojection.is_some() {
            reprojection.prev_pos_round().as_ivec2()
        } else {
            screen_pos.as_ivec2()
        };

        sample_pos += (wnoise.sample_disk() * 32.0).as_ivec2();
        sample_idx += 1;

        let sample_pos = camera.contain(sample_pos);
        let sample_surface = prev_prim_surface_map.get(sample_pos);

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
            prev_reservoirs,
            camera.screen_to_idx(sample_pos),
        );

        if sample.is_empty() {
            continue;
        }

        let sample_pdf = sample.sample.diff_pdf(hit_point, hit_normal);

        sample.clamp_m(1.0);

        if main.merge(&mut wnoise, &sample, sample_pdf) {
            main_pdf = sample_pdf;
        }
    }

    // -------------------------------------------------------------------------

    main.normalize(main_pdf);
    main.write(curr_reservoirs, screen_idx);
}
