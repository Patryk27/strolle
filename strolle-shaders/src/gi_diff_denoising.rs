use strolle_gpu::prelude::*;

const MAX_HISTORY: f32 = 24.0;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 2)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4)] prev_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] samples: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] colors: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 7)] prev_colors: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let reprojection_map = ReprojectionMap::new(reprojection_map);
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let ray = camera.ray(screen_pos);
    let center_surface = surface_map.get(screen_pos);
    let center_point = ray.at(center_surface.depth);
    let center_normal = center_surface.normal;
    let reprojection = reprojection_map.get(screen_pos);

    if center_surface.is_sky() {
        unsafe {
            colors.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    // ---

    let mut previous;
    let history;

    if reprojection.is_some() {
        let sample = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_colors.read(pos), 1.0)
        });

        previous = 2.0 * sample.xyz().extend(1.0);
        history = sample.w;
    } else {
        previous = Vec4::ZERO;
        history = 0.0;
    }

    // ---

    let mut sample_idx = 0;
    let mut sample_angle = bnoise.second_sample().x * 2.0 * PI;
    let mut sample_radius = 0.0;

    let (sample_kernel_t, sample_kernel_b) = Hit::kernel_basis(
        center_normal,
        ray.direction(),
        1.0,
        center_surface.depth,
    );

    let center_sample = samples.read(screen_pos);
    let center_luma = center_sample.xyz().perc_luma();

    while sample_idx < 8 {
        sample_idx += 1;
        sample_angle += GOLDEN_ANGLE;
        sample_radius += lerp(0.006, 0.004, history / MAX_HISTORY);

        let sample_pos = {
            let offset = sample_angle.cos() * sample_kernel_t
                + sample_angle.sin() * sample_kernel_b;

            let offset = offset * sample_radius;

            prev_camera.world_to_screen(center_point + offset)
        };

        if !camera.contains(sample_pos) {
            continue;
        }

        let sample_pos = sample_pos.as_uvec2();
        let sample_surface = prev_surface_map.get(sample_pos);
        let sample_point = prev_camera.ray(sample_pos).at(sample_surface.depth);
        let sample_color = prev_colors.read(sample_pos).xyz();

        if sample_surface.is_sky() {
            continue;
        }

        let sample_normal = sample_surface.normal;

        let sample_weight = {
            let geometry_weight = {
                let dist = sample_normal.dot(center_point - sample_point);
                let norm = (4.0 + history) / (1.0 + center_surface.depth);

                (1.0 - dist.abs() * norm).saturate()
            };

            let normal_weight = {
                let val = center_normal.dot(sample_normal);
                let max = lerp(0.8, 0.95, history / MAX_HISTORY);

                if val <= max {
                    0.0
                } else {
                    (val - max) / (1.0 - max)
                }
            };

            let luma_weight = 1.0;
            // (-(center_luma - sample_color.perc_luma()).abs().sqrt()
            //     * lerp(0.0, 4.0, history / MAX_HISTORY))
            // .exp();

            geometry_weight * normal_weight * luma_weight
        };

        // sample_weight *= lerp(1.0, 0.33, history / MAX_HISTORY);

        if sample_weight > 0.0 {
            previous +=
                (sample_color.xyz() * sample_weight).extend(sample_weight);
        }
    }

    // -------------------------------------------------------------------------

    let sample = center_sample.xyz();

    let out = if history == 0.0 {
        if previous.w == 0.0 {
            sample.extend(1.0)
        } else {
            let previous = previous.xyz() / previous.w;

            (0.5 * sample + 0.5 * previous).extend(2.0)
        }
    } else {
        let history = history + 1.0;
        let previous = previous.xyz() / previous.w;

        // let previous = if previous.luma() > 0.0 {
        //     previous.with_luma(0.9 * previous.luma() + 0.1 * sample.luma())
        // } else {
        //     previous
        // };

        let speed = 1.0 / history;

        previous
            .lerp(sample, speed)
            .extend(history.min(MAX_HISTORY))
    };

    unsafe {
        colors.write(screen_pos, out);
    }
}
