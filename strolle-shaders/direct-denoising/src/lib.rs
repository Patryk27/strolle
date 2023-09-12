#![no_std]

use strolle_gpu::prelude::*;

const MAX_HISTORY: f32 = 8.0;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(push_constant)] params: &PassParams,
    #[spirv(descriptor_set = 0, binding = 0)] blue_noise_tex: TexRgba8f,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] reprojection_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 2)] surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 3)] prev_surface_map: TexRgba32f,
    #[spirv(descriptor_set = 1, binding = 4)] direct_samples: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 5)] direct_colors: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 6)] prev_direct_colors: TexRgba16f,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let reprojection_map = ReprojectionMap::new(reprojection_map);
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);

    if !debug::DIRECT_DENOISING_ENABLED {
        unsafe {
            direct_colors.write(screen_pos, direct_samples.read(screen_pos));
        }

        return;
    }

    // -------------------------------------------------------------------------

    let mut previous;
    let history;

    let surface = surface_map.get(screen_pos);
    let reprojection = reprojection_map.get(screen_pos);

    if reprojection.is_some() {
        let sample = BilinearFilter::reproject(reprojection, move |pos| {
            (prev_direct_colors.read(pos), 1.0)
        });

        previous = (2.0 * sample.xyz()).extend(2.0);
        history = sample.w;
    } else {
        previous = Vec4::ZERO;
        history = 0.0;
    }

    // -------------------------------------------------------------------------

    let mut sample_idx = 0;
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.second_sample().x;

    while sample_idx < 5 {
        sample_idx += 1;
        sample_radius += 0.5;
        sample_angle += GOLDEN_ANGLE;

        let sample_offset =
            vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

        let sample_pos = if reprojection.is_some() {
            reprojection.prev_pos() + sample_offset
        } else {
            screen_pos.as_vec2() + sample_offset
        };

        let sample_reprojection = Reprojection {
            prev_x: sample_pos.x,
            prev_y: sample_pos.y,
            confidence: 1.0,
            validity: u32::MAX,
        };

        let sample =
            BilinearFilter::reproject(sample_reprojection, move |pos| {
                let weight =
                    prev_surface_map.get(pos).evaluate_similarity_to(&surface);

                (prev_direct_colors.read(pos), weight)
            });

        if sample.w > 0.0 {
            previous += (sample.xyz() * sample.w).extend(sample.w);
        }
    }

    // -------------------------------------------------------------------------

    let current = direct_samples.read(screen_pos).xyz();

    let out = if history == 0.0 {
        if previous.w == 0.0 {
            current.extend(1.0)
        } else {
            let previous = previous.xyz() / previous.w;

            (0.5 * current + 0.5 * previous).extend(2.0)
        }
    } else {
        let history = history + 1.0;
        let previous = previous.xyz() / previous.w;
        let speed = 1.0 / history;

        previous
            .lerp(current, speed)
            .extend(history.min(MAX_HISTORY))
    };

    unsafe {
        direct_colors.write(screen_pos, out);
    }
}
