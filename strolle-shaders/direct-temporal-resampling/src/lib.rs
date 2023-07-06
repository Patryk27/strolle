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
        WhiteNoise::new(params.seed, global_id.xy()),
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
    mut wnoise: WhiteNoise,
    camera: &Camera,
    reprojection_map: ReprojectionMap,
    direct_initial_samples: &[Vec4],
    direct_temporal_reservoirs: &mut [Vec4],
    prev_direct_temporal_reservoirs: &[Vec4],
) {
    let screen_idx = camera.screen_to_idx(screen_pos);

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Load sample created in the direct-initial-shading pass.

    let sample = {
        let d0 = unsafe { *direct_initial_samples.get_unchecked(screen_idx) };

        DirectReservoirSample {
            light_id: LightId::new(d0.w.to_bits()),
            light_contribution: d0.xyz(),
        }
    };

    // -------------------------------------------------------------------------
    // Step 2:
    //
    // Try reprojecting reservoir from the previous frame.
    //
    // Note that, as compared to the spatial resampling pass, in here we do not
    // interpolate between nearby reservoirs - that's not necessary because
    // temporal reservoirs are so short-lived and dynamic that any potential
    // filering is simply not observable anyway, so why waste cycles.

    let mut p_hat = sample.p_hat();
    let mut reservoir = DirectReservoir::new(sample, p_hat, params.frame);
    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut prev_reservoir = DirectReservoir::read(
            prev_direct_temporal_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        prev_reservoir.m_sum *= reprojection.confidence;

        let prev_p_hat = prev_reservoir.sample.p_hat();

        // Older reservoirs are worse candidates for resampling because they
        // represent an older state of the world; so if our reservoir is "old",
        // let's gradually decrease its likelyhood of being chosen:
        let prev_age = prev_reservoir.age(params.frame);

        if prev_age > 6 {
            prev_reservoir.m_sum *= 1.0 - ((prev_age - 6) as f32 / 6.0);
        }

        if reservoir.merge(&mut wnoise, &prev_reservoir, prev_p_hat) {
            p_hat = prev_p_hat;
            reservoir.frame = prev_reservoir.frame;
        }
    }

    reservoir.normalize(p_hat, 1000.0, 10.0);
    reservoir.write(direct_temporal_reservoirs, screen_idx);
}
