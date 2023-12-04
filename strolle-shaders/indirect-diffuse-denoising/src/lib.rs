#![no_std]

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
    #[spirv(descriptor_set = 1, binding = 5)]
    indirect_diffuse_samples: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)]
    indirect_diffuse_colors: TexRgba16,
    #[spirv(descriptor_set = 1, binding = 7)]
    prev_indirect_diffuse_colors: TexRgba16,
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

    let surface = surface_map.get(screen_pos);
    let hit_ray = camera.ray(screen_pos);
    let hit_point = hit_ray.at(surface.depth);
    let hit_normal = surface.normal;
    let reprojection = reprojection_map.get(screen_pos);

    if surface.is_sky() {
        unsafe {
            indirect_diffuse_colors.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    // ---

    let mut previous;
    let history;

    if reprojection.is_some() {
        let sample = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_indirect_diffuse_colors.read(pos), 1.0)
        });

        previous = sample.xyz().extend(1.0);
        history = sample.w;
    } else {
        previous = Vec4::ZERO;
        history = 0.0;
    }

    // ---

    let mut sample_idx = 0;
    let mut sample_angle = bnoise.second_sample().x * 2.0 * PI;
    let mut sample_radius = 0.0;

    let (sample_kernel_t, sample_kernel_b) =
        Hit::kernel_basis(hit_normal, hit_ray.direction(), 1.0, surface.depth);

    while sample_idx < 8 {
        sample_idx += 1;
        sample_angle += GOLDEN_ANGLE;
        sample_radius += lerp(0.006, 0.004, history / MAX_HISTORY);

        let sample_pos = {
            let offset = sample_angle.cos() * sample_kernel_t
                + sample_angle.sin() * sample_kernel_b;

            let offset = offset * sample_radius;

            prev_camera.world_to_screen(hit_point + offset)
        };

        if !camera.contains(sample_pos) {
            continue;
        }

        let sample_pos = sample_pos.as_uvec2();
        let sample_color = prev_indirect_diffuse_colors.read(sample_pos);
        let sample_surface = prev_surface_map.get(sample_pos);

        if sample_surface.is_sky() {
            continue;
        }

        let sample_point = prev_camera.ray(sample_pos).at(sample_surface.depth);
        let sample_normal = sample_surface.normal;

        let mut sample_weight = {
            let geometry_weight = {
                let ray = hit_point - sample_point;
                let dist_to_plane = sample_normal.dot(ray);
                let plane_dist_norm = history / (1.0 + surface.depth);

                (1.0 - dist_to_plane.abs() * plane_dist_norm).saturate()
            };

            let normal_weight = {
                let val = hit_normal.dot(sample_normal);
                let max = lerp(0.5, 0.95, history / MAX_HISTORY);

                if val <= max {
                    0.0
                } else {
                    (val - max) / (1.0 - max)
                }
            };

            geometry_weight * normal_weight
        };

        sample_weight *= lerp(1.0, 0.2, history / MAX_HISTORY);

        if sample_weight > 0.0 {
            previous +=
                (sample_color.xyz() * sample_weight).extend(sample_weight);
        }
    }

    // -------------------------------------------------------------------------

    let current = indirect_diffuse_samples.read(screen_pos);

    let out = if history == 0.0 {
        if previous.w == 0.0 {
            current.xyz().extend(1.0)
        } else {
            let previous = previous.xyz() / previous.w;

            (0.5 * current.xyz() + 0.5 * previous).extend(2.0)
        }
    } else {
        let history = history + 1.0;
        let previous = previous.xyz() / previous.w;
        let speed = 1.0 / history;

        previous
            .lerp(current.xyz(), speed)
            .extend(history.min(MAX_HISTORY))
    };

    unsafe {
        indirect_diffuse_colors.write(screen_pos, out);
    }
}
