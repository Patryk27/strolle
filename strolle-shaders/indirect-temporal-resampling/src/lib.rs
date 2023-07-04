#![no_std]

use strolle_gpu::prelude::*;

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
    prev_indirect_temporal_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        ReprojectionMap::new(reprojection_map),
        indirect_initial_samples,
        indirect_temporal_reservoirs,
        prev_indirect_temporal_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    scren_pos: UVec2,
    params: &IndirectTemporalResamplingPassParams,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    indirect_initial_samples: &[Vec4],
    indirect_temporal_reservoirs: &mut [Vec4],
    prev_indirect_temporal_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, scren_pos);
    let screen_idx = camera.screen_to_idx(scren_pos);

    // -------------------------------------------------------------------------

    let d0 =
        unsafe { *indirect_initial_samples.get_unchecked(3 * screen_idx + 0) };

    let d1 =
        unsafe { *indirect_initial_samples.get_unchecked(3 * screen_idx + 1) };

    let d2 =
        unsafe { *indirect_initial_samples.get_unchecked(3 * screen_idx + 2) };

    let sample = IndirectReservoirSample {
        radiance: d0.xyz(),
        hit_point: d1.xyz(),
        sample_point: d2.xyz(),
        sample_normal: Normal::decode(vec2(d0.w, d1.w)),
    };

    let is_sample_valid = d2.w.to_bits() == 1;

    let mut p_hat = sample.p_hat();

    let mut reservoir = if is_sample_valid {
        IndirectReservoir::new(sample, p_hat, params.frame)
    } else {
        IndirectReservoir::empty(params.frame)
    };

    // -------------------------------------------------------------------------

    let reprojection = reprojection_map.get(scren_pos);

    if reprojection.is_some() {
        let mut prev_reservoir = IndirectReservoir::read(
            prev_indirect_temporal_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        let prev_p_hat = prev_reservoir.sample.p_hat();

        // Older reservoirs are worse because they represent older state of the
        // world - and so if we're dealing with an older reservoir, let's reduce
        // its score:
        let prev_age = prev_reservoir.age(params.frame);

        if prev_age > 16 {
            prev_reservoir.m_sum *= 1.0 - ((prev_age - 16) as f32 / 32.0);
        }

        prev_reservoir.m_sum *=
            reprojection.confidence * reprojection.confidence;

        if reservoir.merge(&mut noise, &prev_reservoir, prev_p_hat) {
            p_hat = prev_p_hat;
            reservoir.frame = prev_reservoir.frame;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 10.0, 30.0);
    reservoir.write(indirect_temporal_reservoirs, screen_idx);
}
