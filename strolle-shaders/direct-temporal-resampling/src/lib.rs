#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(push_constant)]
    params: &DirectTemporalResamplingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)]
    reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    direct_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    direct_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    prev_direct_temporal_reservoirs: &[Vec4],
) {
    main_inner(
        global_id.xy(),
        params,
        camera,
        ReprojectionMap::new(reprojection_map),
        direct_initial_samples,
        direct_temporal_reservoirs,
        prev_direct_temporal_reservoirs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    screen_pos: UVec2,
    params: &DirectTemporalResamplingPassParams,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    direct_initial_samples: &[Vec4],
    direct_temporal_reservoirs: &mut [Vec4],
    prev_direct_temporal_reservoirs: &[Vec4],
) {
    let mut noise = Noise::new(params.seed, screen_pos);
    let screen_idx = camera.screen_to_idx(screen_pos);

    let sample = {
        let d0 = unsafe { *direct_initial_samples.get_unchecked(screen_idx) };

        DirectReservoirSample {
            light_id: LightId::new(d0.w.to_bits()),
            light_contribution: d0.xyz(),
        }
    };

    let mut p_hat = sample.p_hat();
    let mut reservoir = DirectReservoir::new(sample, p_hat, params.frame);
    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut prev_reservoir = DirectReservoir::read(
            prev_direct_temporal_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        let prev_p_hat = prev_reservoir.sample.p_hat();
        let prev_age = prev_reservoir.age(params.frame);

        if prev_age > 6 {
            prev_reservoir.m_sum *= 1.0 - ((prev_age - 6) as f32 / 32.0);
        }

        prev_reservoir.m_sum *= reprojection.confidence;

        if reservoir.merge(&mut noise, &prev_reservoir, prev_p_hat) {
            p_hat = prev_p_hat;
            reservoir.frame = prev_reservoir.frame;
        }
    }

    reservoir.normalize(p_hat, 10.0, 30.0);
    reservoir.write(direct_temporal_reservoirs, screen_idx);
}
