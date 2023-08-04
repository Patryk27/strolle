#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 2)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5)]
    indirect_diffuse_samples: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 6)]
    indirect_diffuse_colors: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 7)]
    prev_indirect_diffuse_colors: TexRgba16f,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let reprojection_map = ReprojectionMap::new(reprojection_map);
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);

    // -------------------------------------------------------------------------

    let mut out = Vec4::ZERO;
    let mut out_history = 0.0;

    let screen_surface = surface_map.get(screen_pos);
    let screen_point = camera.ray(screen_pos).target(screen_surface.depth);

    // -------------------------------------------------------------------------

    let screen_reprojection = reprojection_map.get(screen_pos);

    if screen_reprojection.is_some() {
        let default_sample = prev_indirect_diffuse_colors
            .read(screen_reprojection.prev_screen_pos());

        let filter = BilinearFilter::from_reprojection(
            screen_reprojection,
            move |pos| {
                if !camera.contains(pos) {
                    return default_sample;
                }

                let pos = pos.as_uvec2();

                if prev_surface_map
                    .get(pos)
                    .evaluate_similarity_to(&screen_surface)
                    < 0.33
                {
                    return default_sample;
                }

                prev_indirect_diffuse_colors.read(pos)
            },
        );

        out += filter.eval_reprojection(screen_reprojection);

        out_history = out.w;
        out.w = 1.0;
    }

    // -------------------------------------------------------------------------

    let mut sample_idx = 0;
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.second_sample().x;

    while sample_idx < 6 {
        sample_idx += 1;
        sample_radius += 1.66;
        sample_angle += GOLDEN_ANGLE;

        let sample_pos = {
            let delta =
                vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

            screen_pos.as_ivec2() + delta.as_ivec2()
        };

        let sample_pos = camera.contain(sample_pos);
        let sample_reprojection = reprojection_map.get(sample_pos);

        if sample_reprojection.is_none() {
            continue;
        }

        let sample = prev_indirect_diffuse_colors
            .read(sample_reprojection.prev_screen_pos());

        let sample_pos = sample_reprojection.prev_screen_pos();
        let sample_surface = prev_surface_map.get(sample_pos);

        let sample_point =
            prev_camera.ray(sample_pos).target(sample_surface.depth);

        let sample_weight = {
            let normal_weight = screen_surface
                .normal
                .dot(sample_surface.normal)
                .saturate()
                .powf(3.0);

            let distance_weight = {
                let distance = screen_point.distance(sample_point);
                let weight = distance * lerp(0.1, 1.0, out_history * 0.1);

                1.0 - weight.min(1.0)
            };

            normal_weight * distance_weight * sample.w
        };

        out += (sample.xyz() * sample_weight).extend(sample_weight);
    }

    // -------------------------------------------------------------------------

    let out_history = (out_history + 1.0).min(24.0);

    let out = {
        let sample = indirect_diffuse_samples.read(screen_pos).xyz();

        if out.w > 0.0 {
            let speed = 1.0 / (1.0 + out_history);

            lerp(out.xyz() / out.w, sample, speed)
        } else {
            sample
        }
    };

    unsafe {
        indirect_diffuse_colors.write(screen_pos, out.extend(out_history));
    }
}
