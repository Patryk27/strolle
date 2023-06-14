#![no_std]

use spirv_std::glam::{vec2, UVec2, UVec3, Vec3Swizzles, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::spirv;
use strolle_gpu::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &IndirectTemporalResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    indirect_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    indirect_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    past_indirect_temporal_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        ReprojectionMap::new(reprojection_map),
        indirect_initial_samples,
        indirect_temporal_reservoirs,
        past_indirect_temporal_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    params: &IndirectTemporalResamplingPassParams,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    indirect_initial_samples: &[Vec4],
    indirect_temporal_reservoirs: &mut [Vec4],
    past_indirect_temporal_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, global_id);
    let global_idx = camera.half_screen_to_idx(global_id);

    let sample = {
        let d0 = indirect_initial_samples[3 * global_idx];
        let d1 = indirect_initial_samples[3 * global_idx + 1];
        let d2 = indirect_initial_samples[3 * global_idx + 2];

        // Adding some noise to the radiance helps to converge darker areas
        // better:
        let radiance = d0.xyz() + noise.sample_sphere() / 1000.0;

        IndirectReservoirSample {
            radiance,
            hit_point: d1.xyz(),
            sample_point: d2.xyz(),
            sample_normal: Normal::decode(vec2(d0.w, d1.w)),
        }
    };

    let mut p_hat = sample.p_hat();
    let mut reservoir = IndirectReservoir::new(sample, p_hat, params.frame);

    let reprojection =
        reprojection_map.get(upsample(global_id, params.frame - 1));

    if reprojection.is_valid() {
        let mut prev_reservoir = IndirectReservoir::read(
            past_indirect_temporal_reservoirs,
            camera.half_screen_to_idx(reprojection.prev_screen_pos() / 2),
        );

        let prev_p_hat = prev_reservoir.sample.p_hat();

        // TODO taking age into account seems to introduce lots of noise,
        //      investigate why
        let _ = prev_reservoir.age(params.frame);

        prev_reservoir.m_sum *= reprojection.confidence.powi(2);

        if reservoir.merge(&mut noise, &prev_reservoir, prev_p_hat) {
            p_hat = prev_p_hat;
            reservoir.frame = prev_reservoir.frame;
        }
    }

    reservoir.normalize(p_hat, 10.0, 30.0);
    reservoir.write(indirect_temporal_reservoirs, global_idx);
}
