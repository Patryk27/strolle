#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn reproject(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] samples: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 4)] prev_colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 5)] colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 6)] prev_moments: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 7)] moments: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let surface_map = SurfaceMap::new(surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let color;
    let moment;

    let sample = samples.read(screen_pos);
    let sample_luma = sample.xyz().luminance();
    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let prev_color = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_colors.read(pos), 1.0)
        });

        let prev_moment = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_moments.read(pos), 1.0)
        });

        let prev_color = prev_color.xyz();
        let prev_history = prev_moment.x;
        let prev_m1 = prev_moment.y;
        let prev_m2 = prev_moment.z;

        let new_history = prev_history + 1.0;
        let color_a = (1.0 / new_history).max(0.2);
        let moment_a = (1.0 / new_history).max(0.2);

        let new_m1 = (1.0 - moment_a) * prev_m1 + moment_a * sample_luma;

        let new_m2 =
            (1.0 - moment_a) * prev_m2 + moment_a * sample_luma * sample_luma;

        color = (1.0 - color_a) * prev_color + color_a * sample.xyz();
        moment = vec3(new_history, new_m1, new_m2);
    } else {
        color = sample.xyz();
        moment = vec3(1.0, sample_luma, sample_luma * sample_luma);
    }

    let moment = moment.extend(surface_map.get(screen_pos).depth);

    unsafe {
        colors.write(screen_pos, color.extend(0.0));
        moments.write(screen_pos, moment);
    }
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn estimate_variance(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] colors: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] moments: TexRgba32,
) {
    let screen_pos = global_id.xy();

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let color = colors.read(screen_pos).xyz();
    let moment = moments.read(screen_pos);
    let mut variance = (moment.z - moment.y * moment.y).abs();

    if moment.x == 1.0 {
        variance = 1.0;
    }

    unsafe {
        colors.write(screen_pos, color.extend(variance));
    }
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn wavelet(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &DirectDenoisingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] input: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 3)] output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let surface_map = SurfaceMap::new(surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    const NORMAL_PHI: f32 = 32.0;
    const LUMA_PHI: f32 = 4.0;

    let kernel = [1.0 / 16.0, 1.0 / 4.0, 3.0 / 8.0, 1.0 / 4.0, 1.0 / 16.0];

    let center_sample = input.read(screen_pos);
    let center_color = center_sample.xyz();
    let center_variance = center_sample.w;
    let center_luma = center_color.luminance();

    let center_surface = surface_map.get(screen_pos);
    let center_normal = center_surface.normal;
    let center_depth = center_surface.depth;

    let center_variance_avg = {
        let kernel = [1.0 / 4.0, 1.0 / 8.0, 1.0 / 16.0];
        let mut delta = ivec2(-1, -1);
        let mut sum = 0.0;

        loop {
            let sample_pos = screen_pos.as_ivec2() + delta;

            if camera.contains(sample_pos) {
                let sample_weight =
                    kernel[(delta.x.abs() + delta.y.abs()) as usize];

                let sample_variance = input.read(sample_pos.as_uvec2()).w;

                sum += sample_weight * sample_variance;
            }

            // ---

            delta.x += 1;

            if delta.x == 2 {
                delta.x = -1;
                delta.y += 1;

                if delta.y == 2 {
                    break;
                }
            }
        }

        sum.sqrt()
    };

    if center_variance_avg < 0.00001 {
        unsafe {
            output.write(screen_pos, center_color.extend(center_variance));
        }

        return;
    }

    let mut sum_weights = kernel[2] * kernel[2];
    let mut sum_color = sum_weights * center_color;
    let mut sum_variance = sum_weights * sum_weights * center_variance;

    let mut delta = ivec2(-2, -2);

    loop {
        let sample_pos = screen_pos.as_ivec2() + delta * (params.stride as i32);

        if camera.contains(sample_pos) && delta != ivec2(0, 0) {
            let sample_pos = sample_pos.as_uvec2();

            let sample = input.read(sample_pos);
            let sample_color = sample.xyz();
            let sample_luma = sample_color.luminance();
            let sample_variance = sample.w;

            let sample_surface = surface_map.get(sample_pos);
            let sample_normal = sample_surface.normal;
            let sample_depth = sample_surface.depth;

            if sample_depth != 0.0 {
                let mut weight = {
                    let luma_weight = (sample_luma - center_luma).abs()
                        / (LUMA_PHI * center_variance_avg);

                    let depth_weight = {
                        let leeway = center_depth * 0.2;
                        let diff = (sample_depth - center_depth).abs();

                        if diff >= leeway {
                            0.0
                        } else {
                            1.0 - diff / leeway
                        }
                    };

                    let normal_weight = sample_normal
                        .dot(center_normal)
                        .max(0.0)
                        .powf(NORMAL_PHI);

                    (0.0 - luma_weight).exp()
                        * depth_weight.max(0.0)
                        * normal_weight.max(0.0)
                };

                weight *= {
                    let kx = kernel[(2 + delta.x) as usize];
                    let ky = kernel[(2 + delta.y) as usize];

                    kx * ky
                };

                sum_weights += weight;
                sum_color += weight * sample_color;
                sum_variance += weight * weight * sample_variance;
            }
        }

        // ---

        delta.x += 1;

        if delta.x == 3 {
            delta.x = -2;
            delta.y += 1;

            if delta.y == 3 {
                break;
            }
        }
    }

    let out_color = sum_color / sum_weights;
    let out_variance = sum_variance / (sum_weights * sum_weights);

    unsafe {
        output.write(screen_pos, out_color.extend(out_variance));
    }
}
