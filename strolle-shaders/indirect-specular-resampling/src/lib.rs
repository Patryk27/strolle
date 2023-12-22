#![no_std]

use spirv_std::arch::IndexUnchecked;
use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d1: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    indirect_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_specular_curr_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_specular_prev_reservoirs: &[Vec4],
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
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // ---

    let mut main = IndirectReservoir::default();
    let mut main_pdf = 0.0;

    let d0 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx) };
    let d1 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx + 1) };
    let d2 = unsafe { *indirect_samples.index_unchecked(3 * screen_idx + 2) };

    if d0.w.to_bits() == 1 {
        let sample = IndirectReservoirSample {
            radiance: d1.xyz(),
            direct_point: d0.xyz(),
            indirect_point: d2.xyz(),
            indirect_normal: Normal::decode(vec2(d1.w, d2.w)),
            frame: params.frame,
        };

        main_pdf = sample.temporal_pdf();
        main.update(&mut wnoise, sample, main_pdf);
    }

    // ---

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() && !hit.gbuffer.is_mirror() {
        let sample = IndirectReservoir::read(
            indirect_specular_prev_reservoirs,
            camera.screen_to_idx(reprojection.prev_pos_round()),
        );

        if sample.sample.is_within_specular_lobe_of(&hit) {
            let sample_pdf = sample.sample.temporal_pdf();

            if main.merge(&mut wnoise, &sample, sample_pdf) {
                main_pdf = sample_pdf;
            }
        }
    }

    // ---

    main.normalize(main_pdf);
    main.clamp_m(32.0);
    main.write(indirect_specular_curr_reservoirs, screen_idx);
}
