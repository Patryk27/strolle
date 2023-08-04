#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_hits: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 4)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    indirect_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 6, storage_buffer)]
    indirect_specular_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 7, storage_buffer)]
    prev_indirect_specular_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------

    let direct_hit = Hit::from_direct(
        camera.ray(screen_pos),
        direct_hits.read(screen_pos).xyz(),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------

    let mut p_hat = Default::default();
    let mut reservoir = IndirectReservoir::default();

    let sample_pos =
        if IndirectReservoir::expects_specular_sample(screen_pos, params.frame)
        {
            screen_pos
        } else {
            screen_pos + uvec2(1, 0)
        };

    let can_use_sample = sample_pos.x < camera.screen_size().x;

    if can_use_sample {
        let sample_idx = camera.screen_to_idx(sample_pos);

        let d0 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 0) };
        let d1 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 1) };
        let d2 = unsafe { *indirect_samples.get_unchecked(3 * sample_idx + 2) };

        if d0.w.to_bits() == 1 {
            let sample = IndirectReservoirSample {
                radiance: d1.xyz(),
                hit_point: d0.xyz(),
                sample_point: d2.xyz(),
                sample_normal: Normal::decode(vec2(d1.w, d2.w)),
                frame: params.frame,
            };

            p_hat = sample.temporal_p_hat();

            reservoir.add(&mut wnoise, sample, p_hat);
        }
    }

    // -------------------------------------------------------------------------

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let rhs = IndirectReservoir::read(
            prev_indirect_specular_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        if rhs.sample.specular_brdf(&direct_hit) > 0.0 {
            let rhs_p_hat = rhs.sample.temporal_p_hat();

            if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
                p_hat = rhs_p_hat;
            }
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 20.0);
    reservoir.write(indirect_specular_reservoirs, screen_idx);
}
