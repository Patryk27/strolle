#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    direct_initial_samples: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    direct_temporal_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)]
    prev_direct_temporal_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Load sample created in the direct-initial-shading pass.

    let sample = {
        let d0 = unsafe {
            *direct_initial_samples.get_unchecked(2 * screen_idx + 0)
        };

        let d1 = unsafe {
            *direct_initial_samples.get_unchecked(2 * screen_idx + 1)
        };

        DirectReservoirSample {
            light_id: LightId::new(d0.w.to_bits()),
            light_contribution: d0.xyz(),
            hit_point: d1.xyz(),
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
        let mut rhs = DirectReservoir::read(
            prev_direct_temporal_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        rhs.m_sum *= reprojection.confidence;

        let rhs_p_hat = rhs.sample.p_hat();

        // Older reservoirs are worse candidates for resampling because they
        // represent an older state of the world; so if our reservoir is "old",
        // let's gradually decrease its likelyhood of being chosen:
        let rhs_age = rhs.age(params.frame);

        if rhs_age > 6 {
            rhs.m_sum *= 1.0 - ((rhs_age - 6) as f32 / 6.0);
        }

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
            p_hat = rhs_p_hat;
            reservoir.frame = rhs.frame;
        }
    }

    reservoir.normalize(p_hat, 1000.0, 10.0);
    reservoir.write(direct_temporal_reservoirs, screen_idx);
}
