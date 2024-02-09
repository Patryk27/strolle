use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] prim_gbuffer_d1: TexRgba32,
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
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
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

    // ---

    let mut main = GiReservoir::default();
    let mut main_pdf = 0.0;

    if got_checkerboard_at(screen_pos, params.frame) {
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

            main_pdf = sample.spec_pdf();
            main.update(&mut wnoise, sample, main_pdf);
        }
    }

    // ---

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() && !hit.gbuffer.is_mirror() {
        let sample = GiReservoir::read(
            prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        if sample.sample.is_within_spec_lobe_of(&hit) {
            let sample_pdf = sample.sample.spec_pdf();

            if main.merge(&mut wnoise, &sample, sample_pdf) {
                main_pdf = sample_pdf;
            }
        }
    }

    // ---

    main.normalize(main_pdf);
    main.clamp_m(32.0);
    main.write(curr_reservoirs, screen_idx);
}
