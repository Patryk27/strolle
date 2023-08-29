#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    direct_temporal_reservoirs: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 6, storage_buffer)]
    direct_spatial_reservoirs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 7, storage_buffer)]
    prev_direct_spatial_reservoirs: &[Vec4],
) {
    let screen_pos = global_id.xy();
    let screen_idx = camera.screen_to_idx(screen_pos);
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let mut wnoise = WhiteNoise::new(params.seed, screen_pos);
    let surface_map = SurfaceMap::new(surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    // -------------------------------------------------------------------------

    let surface = surface_map.get(screen_pos);
    let mut reservoir = DirectReservoir::default();

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // -------------------------------------------------------------------------
    // Step 1:
    //
    // Try reprojecting reservoir from the previous frame; as an extra, we are
    // using bilinear filtering to reduce smearing in case of camera rotations
    // and movements.

    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let mut rhs = DirectReservoir::read(
            prev_direct_spatial_reservoirs,
            camera.screen_to_idx(reprojection.prev_screen_pos()),
        );

        let sample = BilinearFilter::reproject(reprojection, move |pos| {
            let reservoir = DirectReservoir::read(
                prev_direct_spatial_reservoirs,
                camera.screen_to_idx(pos),
            );

            if reservoir.sample.light_id == rhs.sample.light_id {
                (reservoir.sample.light_radiance.extend(reservoir.w), 1.0)
            } else {
                (Vec4::ZERO, 0.0)
            }
        });

        rhs.sample.light_radiance = sample.xyz();
        rhs.w = sample.w;
        rhs.m_sum *= reprojection.confidence;

        reservoir.merge(&mut wnoise, &rhs, rhs.sample.p_hat());
    }

    // -------------------------------------------------------------------------
    // Step 2:
    //
    // Analyze our screen-space neighbourhood and try to incorporate samples
    // from temporal reservoirs around us into our current reservoir.
    //
    // To get a good coverage, we follow a spiral pattern driven by blue noise:
    //
    // ```
    //         5
    //
    //
    //     1
    // 4   C 2
    //
    //     3
    //
    // (C - our current sample; 1..5 - order of looking at neighbours)
    // ```

    let mut p_hat = reservoir.sample.p_hat();
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.second_sample().x;
    let mut max_sample_radius = lerp(12.0, 6.0, reservoir.m_sum / 250.0);

    while sample_radius < max_sample_radius {
        let rhs_pos = screen_pos.as_vec2()
            + vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

        let rhs_pos = rhs_pos.as_ivec2();

        sample_radius += 1.0;
        sample_angle += GOLDEN_ANGLE;

        if !camera.contains(rhs_pos) {
            continue;
        }

        let rhs_pos = rhs_pos.as_uvec2();

        // TODO implement a screen-space occlusion check
        let mut rhs_similarity = surface_map
            .evaluate_similarity_between(screen_pos, surface, rhs_pos);

        if rhs_similarity < 0.5 {
            continue;
        }

        // If the surface we're looking at has very similar characteristics to
        // our center surface, there's a good chance we're looking at a wall or
        // a floor - i.e. a continuous, long surface.
        //
        // In this case, we'd like to extend the search radius to gather more
        // samples from far-away reservoirs to reduce boiling artifacts.
        if rhs_similarity > 0.75 {
            sample_radius += 1.0;
            max_sample_radius += 0.75;
        }

        let rhs = DirectReservoir::read(
            direct_temporal_reservoirs,
            camera.screen_to_idx(rhs_pos),
        );

        // If we're starved for samples, accept samples from far-away
        // reservoirs; otherwise attenuate them to avoid introducing noise.
        rhs_similarity *= (1.0 - rhs.sample.hit_point.distance(hit.point))
            .max(0.1 + 0.5 - 0.5 * reservoir.m_sum.min(500.0) / 500.0);

        let rhs_p_hat = rhs.sample.p_hat();

        if reservoir.merge(&mut wnoise, &rhs, rhs_p_hat * rhs_similarity) {
            p_hat = rhs_p_hat;
        }
    }

    // -------------------------------------------------------------------------

    reservoir.normalize(p_hat, 1000.0, 500.0);
    reservoir.write(direct_spatial_reservoirs, screen_idx);
}
