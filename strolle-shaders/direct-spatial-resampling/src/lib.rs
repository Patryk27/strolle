#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2, storage_buffer)]
    direct_curr_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    direct_next_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let surface_map = SurfaceMap::new(surface_map);

    // -------------------------------------------------------------------------

    let mut reservoir = DirectReservoir::default();
    let reservoir_confidence = (reservoir.m_sum / 500.0).min(1.0);

    let surface = surface_map.get(screen_pos);

    if surface.depth == 0.0 {
        reservoir.write(direct_next_reservoirs, screen_idx);
        return;
    }

    let mut p_hat = reservoir.sample.p_hat();
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.first_sample().x;

    while sample_radius < 5.0 {
        sample_radius += 1.0;
        sample_angle += GOLDEN_ANGLE;

        let rhs_pos = if sample_radius > 1.0 {
            let rhs_offset =
                vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

            let rhs_pos = screen_pos.as_vec2() + rhs_offset;

            camera.contain(rhs_pos.as_ivec2())
        } else {
            screen_pos
        };

        if sample_radius > 1.0 {
            let rhs_surface = surface_map.get(rhs_pos);

            let max_depth_diff =
                lerp(0.2, 0.05, reservoir_confidence) * surface.depth;

            let max_normal_diff = lerp(0.8, 0.9, reservoir_confidence);

            if (rhs_surface.depth - surface.depth).abs() > max_depth_diff {
                continue;
            }

            if rhs_surface.normal.dot(surface.normal) < max_normal_diff {
                continue;
            }
        }

        let rhs = DirectReservoir::read(
            direct_curr_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat();

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
            p_hat = rhs_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 1000.0);
    reservoir.write(direct_next_reservoirs, screen_idx);
}
