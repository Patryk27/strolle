use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn reproject(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 0, binding = 2)] reprojection_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 0)] prev_colors: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 1)] prev_moments: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] samples: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 3)] colors: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 4)] moments: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let prim_surface_map = SurfaceMap::new(prim_surface_map);
    let reprojection_map = ReprojectionMap::new(reprojection_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    if prim_surface_map.get(screen_pos).is_sky() {
        unsafe {
            colors.write(screen_pos, samples.read(screen_pos));
        }

        return;
    }

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

        let curr_color = sample.xyz();
        let curr_history = (prev_history + 1.0).min(5.0);
        let curr_m1 = sample_luma;
        let curr_m2 = sample_luma * sample_luma;

        let alpha = 1.0 / curr_history;

        color = lerp(prev_color, curr_color, alpha);

        moment = vec3(
            curr_history,
            lerp(prev_m1, curr_m1, alpha),
            lerp(prev_m2, curr_m2, alpha),
        );
    } else {
        color = sample.xyz();
        moment = vec3(1.0, sample_luma, sample_luma * sample_luma);
    }

    unsafe {
        colors.write(screen_pos, color.extend(0.0));
        moments.write(screen_pos, moment.extend(0.0));
    }
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn estimate_variance(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 0)] di_colors: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 1)] di_moments: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 2)] di_output: TexRgba32,
    #[spirv(descriptor_set = 2, binding = 0)] gi_colors: TexRgba32,
    #[spirv(descriptor_set = 2, binding = 1)] gi_moments: TexRgba32,
    #[spirv(descriptor_set = 2, binding = 2)] gi_output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let center_surface = prim_surface_map.get(screen_pos);

    let center_di_color = di_colors.read(screen_pos);
    let center_di_luma = center_di_color.xyz().luminance();
    let center_di_moment = di_moments.read(screen_pos);
    let center_di_var;

    let center_gi_color = gi_colors.read(screen_pos);
    let center_gi_luma = center_gi_color.xyz().luminance();
    let center_gi_moment = gi_moments.read(screen_pos);
    let center_gi_var;

    if center_surface.is_sky() {
        unsafe {
            di_output.write(screen_pos, center_di_color);
            gi_output.write(screen_pos, center_gi_color);
        }

        return;
    }

    if center_di_moment.x >= 4.0 {
        center_di_var =
            center_di_moment.z - center_di_moment.y * center_di_moment.y;

        center_gi_var =
            center_gi_moment.z - center_gi_moment.y * center_gi_moment.y;
    } else {
        let mut sum_di = Vec3::ZERO;
        let mut sum_gi = Vec3::ZERO;
        let mut sample_offset = ivec2(-3, -3);

        loop {
            let sample_pos = screen_pos.as_ivec2() + sample_offset;

            if camera.contains(sample_pos) {
                let sample_pos = sample_pos.as_uvec2();
                let sample_surface = prim_surface_map.get(sample_pos);

                if !sample_surface.is_sky() {
                    let sample_di_color = di_colors.read(sample_pos);
                    let sample_di_luma = sample_di_color.xyz().luminance();

                    let sample_di_weight = eval_sample_weight(
                        center_di_luma,
                        center_surface,
                        sample_di_luma,
                        sample_surface,
                        1.0,
                    );

                    sum_di += vec3(
                        sample_di_luma,
                        sample_di_luma * sample_di_luma,
                        1.0,
                    ) * Vec3::splat(sample_di_weight);

                    // ---

                    let sample_gi_color = gi_colors.read(sample_pos);
                    let sample_gi_luma = sample_gi_color.xyz().luminance();

                    let sample_gi_weight = eval_sample_weight(
                        center_gi_luma,
                        center_surface,
                        sample_gi_luma,
                        sample_surface,
                        1.0,
                    );

                    sum_gi += vec3(
                        sample_gi_luma,
                        sample_gi_luma * sample_gi_luma,
                        1.0,
                    ) * Vec3::splat(sample_gi_weight);
                }
            }

            // ---

            sample_offset.x += 1;

            if sample_offset.x == 4 {
                sample_offset.x = -3;
                sample_offset.y += 1;

                if sample_offset.y == 4 {
                    break;
                }
            }
        }

        center_di_var = {
            let m1 = sum_di.x / sum_di.z;
            let m2 = sum_di.y / sum_di.z;

            (m2 - m1 * m1).abs().sqrt() * 4.0
        };

        center_gi_var = {
            let m1 = sum_gi.x / sum_gi.z;
            let m2 = sum_gi.y / sum_gi.z;

            (m2 - m1 * m1).abs().sqrt() * 4.0
        };
    };

    let center_di_var = center_di_var.max(0.0);
    let center_gi_var = center_gi_var.max(0.0);

    unsafe {
        di_output
            .write(screen_pos, center_di_color.xyz().extend(center_di_var));

        gi_output
            .write(screen_pos, center_gi_color.xyz().extend(center_gi_var));
    }
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn wavelet(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &FrameDenoisingPassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8,
    #[spirv(descriptor_set = 0, binding = 1, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 2)] prim_surface_map: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 0)] di_input: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 1)] di_output: TexRgba32,
    #[spirv(descriptor_set = 2, binding = 0)] gi_input: TexRgba32,
    #[spirv(descriptor_set = 2, binding = 1)] gi_output: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let prim_surface_map = SurfaceMap::new(prim_surface_map);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let center_surface = prim_surface_map.get(screen_pos);

    let center_di = di_input.read(screen_pos);
    let center_di_color = center_di.xyz();
    let center_di_var = center_di.w;
    let center_di_luma = center_di_color.luminance();

    if center_surface.is_sky() {
        unsafe {
            di_output.write(screen_pos, center_di_color.extend(center_di_var));
        }

        return;
    }

    let center_gi = gi_input.read(screen_pos);
    let center_gi_color = center_gi.xyz();
    let center_gi_var = center_gi.w;
    let center_gi_luma = center_gi_color.luminance();

    // ---

    let (center_di_var_avg, center_gi_var_avg) = {
        let kernel = [1.0 / 4.0, 1.0 / 8.0, 1.0 / 16.0];
        let mut sum = vec2(0.0, 0.0);
        let mut sample_offset = ivec2(-1, -1);

        loop {
            let sample_pos = screen_pos.as_ivec2() + sample_offset;

            if camera.contains(sample_pos) {
                let sample_weight = kernel
                    [(sample_offset.x.abs() + sample_offset.y.abs()) as usize];

                let sample_di_var = di_input.read(sample_pos.as_uvec2()).w;
                let sample_gi_var = gi_input.read(sample_pos.as_uvec2()).w;

                sum += vec2(sample_di_var, sample_gi_var) * sample_weight;
            }

            // ---

            sample_offset.x += 1;

            if sample_offset.x == 2 {
                sample_offset.x = -1;
                sample_offset.y += 1;

                if sample_offset.y == 2 {
                    break;
                }
            }
        }

        (sum.x, sum.y)
    };

    let luma_sigma_di = lerp(4.0, 0.5, center_di_var_avg.sqrt());
    let luma_sigma_gi = lerp(1.0, 0.5, center_gi_var_avg.sqrt());

    let jitter =
        ((bnoise.second_sample() - 0.5) * (params.stride as f32) * 0.33)
            .as_ivec2();

    let mut sum_di_weights = 1.0;
    let mut sum_di_color = center_di_color;
    let mut sum_di_var = center_di_var;

    let mut sum_gi_weights = 1.0;
    let mut sum_gi_color = center_gi_color;
    let mut sum_gi_var = center_gi_var;

    let mut sample_offset = ivec2(-1, -1);

    loop {
        let sample_pos = screen_pos.as_ivec2()
            + sample_offset * (params.stride as i32)
            + jitter;

        if camera.contains(sample_pos) && sample_offset != ivec2(0, 0) {
            let sample_pos = sample_pos.as_uvec2();
            let sample_surface = prim_surface_map.get(sample_pos);

            if !sample_surface.is_sky() {
                let sample_di = di_input.read(sample_pos);
                let sample_di_color = sample_di.xyz();
                let sample_di_var = sample_di.w;
                let sample_di_luma = sample_di_color.luminance();

                let sample_di_weight = eval_sample_weight(
                    center_di_luma,
                    center_surface,
                    sample_di_luma,
                    sample_surface,
                    luma_sigma_di,
                );

                if sample_di_weight > 0.0 {
                    sum_di_weights += sample_di_weight;
                    sum_di_color += sample_di_weight * sample_di_color;

                    sum_di_var +=
                        sample_di_weight * sample_di_weight * sample_di_var;
                }

                // ---

                let sample_gi = gi_input.read(sample_pos);
                let sample_gi_color = sample_gi.xyz();
                let sample_gi_var = sample_gi.w;
                let sample_gi_luma = sample_gi_color.luminance();

                let sample_gi_weight = eval_sample_weight(
                    center_gi_luma,
                    center_surface,
                    sample_gi_luma,
                    sample_surface,
                    luma_sigma_gi,
                );

                if sample_gi_weight > 0.0 {
                    sum_gi_weights += sample_gi_weight;
                    sum_gi_color += sample_gi_weight * sample_gi_color;

                    sum_gi_var +=
                        sample_gi_weight * sample_gi_weight * sample_gi_var;
                }
            }
        }

        // ---

        sample_offset.x += 1;

        if sample_offset.x == 2 {
            sample_offset.x = -1;
            sample_offset.y += 1;

            if sample_offset.y == 2 {
                break;
            }
        }
    }

    let out_di_color = sum_di_color / sum_di_weights;
    let out_di_var = sum_di_var / (sum_di_weights * sum_di_weights);

    let out_gi_color = sum_gi_color / sum_gi_weights;
    let out_gi_var = sum_gi_var / (sum_gi_weights * sum_gi_weights);

    unsafe {
        di_output.write(screen_pos, out_di_color.extend(out_di_var));
        gi_output.write(screen_pos, out_gi_color.extend(out_gi_var));
    }
}

fn eval_sample_weight(
    center_luma: f32,
    center_surface: Surface,
    sample_luma: f32,
    sample_surface: Surface,
    luma_sigma: f32,
) -> f32 {
    let luma_weight =
        (center_luma - sample_luma).abs().sqrt() * luma_sigma.max(0.0);

    let depth_weight = {
        let leeway = center_surface.depth * 0.2;
        let diff = (sample_surface.depth - center_surface.depth).abs();

        if diff >= leeway {
            0.0
        } else {
            1.0 - diff / leeway
        }
    };

    let normal_weight = sample_surface
        .normal
        .dot(center_surface.normal)
        .max(0.0)
        .powf(32.0);

    (-luma_weight).exp() * depth_weight * normal_weight
}
