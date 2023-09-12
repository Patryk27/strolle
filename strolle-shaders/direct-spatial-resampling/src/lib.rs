#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    direct_curr_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_next_reservoirs: &mut [Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let surface_map = SurfaceMap::new(surface_map);
    let lights = LightsView::new(lights);

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    if hit.is_none() {
        DirectReservoir::default().write(direct_next_reservoirs, screen_idx);
        return;
    }

    let surface = hit.as_surface();

    // ---

    let mut reservoir = DirectReservoir::default();
    let mut reservoir_p_hat = 0.0;
    let mut sample_idx = 0;
    let mut sample_radius = 0.0;
    let mut sample_angle = 2.0 * PI * bnoise.first_sample().x;

    while sample_idx < 6 {
        sample_idx += 1;
        sample_radius += 1.33;
        sample_angle += GOLDEN_ANGLE;

        let rhs_pos = if sample_idx > 1 {
            let rhs_offset =
                vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

            let rhs_pos = screen_pos.as_vec2() + rhs_offset;

            camera.contain(rhs_pos.as_ivec2())
        } else {
            screen_pos
        };

        if sample_idx > 1 {
            let rhs_surface = surface_map.get(rhs_pos);

            if rhs_surface.is_sky() {
                continue;
            }

            if (rhs_surface.depth - surface.depth).abs() > 0.2 * surface.depth {
                continue;
            }

            if rhs_surface.normal.dot(surface.normal) < 0.8 {
                continue;
            }
        }

        let rhs = DirectReservoir::read(
            direct_curr_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        let rhs_p_hat = rhs.sample.p_hat(lights, hit);

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat) {
            reservoir_p_hat = rhs_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(reservoir_p_hat);
    reservoir.clamp_w(10.0);
    reservoir.write(direct_next_reservoirs, screen_idx);
}
