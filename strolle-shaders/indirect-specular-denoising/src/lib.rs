#![no_std]

use strolle_gpu::prelude::*;

const MAX_HISTORY: f32 = 26.0;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 2)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)] direct_gbuffer_d0: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 5)] direct_gbuffer_d1: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 6)]
    indirect_specular_samples: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 7)]
    indirect_specular_colors: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 8)]
    prev_indirect_specular_colors: TexRgba16f,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let reprojection_map = ReprojectionMap::new(reprojection_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);

    if !debug::INDIRECT_SPECULAR_DENOISING_ENABLED {
        unsafe {
            indirect_specular_colors
                .write(screen_pos, indirect_specular_samples.read(screen_pos));
        }

        return;
    }

    // -------------------------------------------------------------------------

    let hit = Hit::new(
        camera.ray(screen_pos),
        GBufferEntry::unpack([
            direct_gbuffer_d0.read(screen_pos),
            direct_gbuffer_d1.read(screen_pos),
        ]),
    );

    // Optimization: if the hit-surface is purely diffuse, don't bother reading
    // the specular samples (which will be all black anyway).
    if hit.gbuffer.is_pure_diffuse() {
        unsafe {
            indirect_specular_colors.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    // -------------------------------------------------------------------------

    let mut previous;
    let history;

    let surface = hit.as_surface();
    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let sample = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_indirect_specular_colors.read(pos), 1.0)
        });

        // TODO should (probably) incorporate motion vector
        let reprojectability = {
            fn ndf_01(roughness: f32, n_dot_h: f32) -> f32 {
                let a2 = roughness * roughness;
                let denom_sqrt = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;

                a2 * a2 / (denom_sqrt * denom_sqrt)
            }

            let curr_dir = (hit.point - camera.approx_origin()).normalize();

            let prev_dir =
                (hit.point - prev_camera.approx_origin()).normalize();

            ndf_01(
                hit.gbuffer.roughness.max(0.1),
                curr_dir.dot(prev_dir).saturate(),
            )
            .saturate()
            .powf(16.0)
        };

        previous = sample.xyz().extend(1.0);
        history = sample.w * reprojectability;
    } else {
        previous = Vec4::ZERO;
        history = 0.0;
    };

    let (kernel_t, kernel_b) =
        hit.specular_kernel_basis(2.0 * hit.gbuffer.roughness.sqrt());

    // TODO should (probably) depend on the angle
    let max_history = (MAX_HISTORY * hit.gbuffer.roughness.sqrt()).ceil();

    // -------------------------------------------------------------------------

    let mut sample_idx = 0;
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.first_sample().y;

    let mut previous_aabb_min = Vec3::MAX;
    let mut previous_aabb_max = Vec3::MIN;

    if history > 0.0 {
        previous_aabb_min = previous_aabb_min.min(previous.xyz());
        previous_aabb_max = previous_aabb_max.max(previous.xyz());
    }

    while sample_idx < 5 {
        sample_idx += 1;
        sample_radius += lerp(0.03, 0.01, history / max_history);
        sample_angle += GOLDEN_ANGLE;

        let sample_offset =
            kernel_t * sample_angle.cos() + kernel_b * sample_angle.sin();

        let sample_pos = hit.point + sample_offset * sample_radius;
        let sample_pos = prev_camera.world_to_screen(sample_pos);

        if !prev_camera.contains(sample_pos.as_ivec2()) {
            continue;
        }

        let sample_pos = sample_pos.as_uvec2();
        let sample_surface = prev_surface_map.get(sample_pos);
        let sample_color = prev_indirect_specular_colors.read(sample_pos).xyz();
        let sample_weight = sample_surface.evaluate_similarity_to(&surface);

        if sample_weight > 0.0 {
            previous += (sample_color * sample_weight).extend(sample_weight);
            previous_aabb_min = previous_aabb_min.min(sample_color);
            previous_aabb_max = previous_aabb_max.max(sample_color);
        }
    }

    // -------------------------------------------------------------------------

    let current = indirect_specular_samples.read(screen_pos);

    let mut current = if history == 0.0 {
        current.xyz()
    } else {
        let current_clipped =
            current.xyz().clip(previous_aabb_min, previous_aabb_max);

        current_clipped.lerp(current.xyz(), current.w)
    };

    // -------------------------------------------------------------------------

    let out = if previous.w == 0.0 {
        current.extend(1.0)
    } else {
        let previous = previous.xyz() / previous.w;
        let speed = 1.0 / (1.0 + history);

        // If our pixel has accumulated some history, we can utilize its
        // neighbourhood to perform a basic firefly rejection
        if history >= 4.0 {
            let average = 0.5 * previous_aabb_min + 0.5 * previous_aabb_max;
            let average_luminance = average.luminance();

            if current.luminance() >= 4.0 * average_luminance {
                // TODO not really correct, it rejects valid samples around the
                //      light sources
                current = average;
            }
        }

        previous
            .lerp(current, speed)
            .extend((history + 1.0).min(max_history))
    };

    unsafe {
        indirect_specular_colors.write(screen_pos, out);
    }
}
