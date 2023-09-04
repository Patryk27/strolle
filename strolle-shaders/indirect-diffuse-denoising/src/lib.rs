#![no_std]

use strolle_gpu::prelude::*;

const MAX_HISTORY: f32 = 24.0;

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
    #[spirv(descriptor_set = 1, binding = 4)]
    indirect_diffuse_samples: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 5)]
    indirect_diffuse_colors: TexRgba16f,
    #[spirv(descriptor_set = 1, binding = 6)]
    prev_indirect_diffuse_colors: TexRgba16f,
) {
    let screen_pos = global_id.xy();
    let bnoise = BlueNoise::new(blue_noise_tex, screen_pos, params.frame);
    let reprojection_map = ReprojectionMap::new(reprojection_map);
    let surface_map = SurfaceMap::new(surface_map);
    let prev_surface_map = SurfaceMap::new(prev_surface_map);

    if !debug::INDIRECT_DIFFUSE_DENOISING_ENABLED {
        unsafe {
            indirect_diffuse_colors
                .write(screen_pos, indirect_diffuse_samples.read(screen_pos));
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
            (prev_indirect_diffuse_colors.read(pos), 1.0)
        });

        previous = sample.xyz().extend(1.0);
        history = sample.w;
    } else {
        previous = Vec4::ZERO;
        history = 0.0;
    }

    // -------------------------------------------------------------------------

    let mut sample_idx = 0;
    let mut sample_radius = 0.0f32;
    let mut sample_angle = 2.0 * PI * bnoise.second_sample().y;

    while sample_idx < 5 {
        sample_idx += 1;
        sample_radius += 1.66;
        sample_angle += GOLDEN_ANGLE;

        let sample_offset =
            vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

        let sample_pos = if reprojection.is_some() {
            reprojection.prev_pos().as_ivec2() + sample_offset.as_ivec2()
        } else {
            screen_pos.as_ivec2() + sample_offset.as_ivec2()
        };

        let sample_pos = camera.contain(sample_pos);
        let sample_surface = prev_surface_map.get(sample_pos);
        let sample_color = prev_indirect_diffuse_colors.read(sample_pos);

        let sample_weight = sample_surface.evaluate_similarity_to(&surface);

        previous += (sample_color.xyz() * sample_weight).extend(sample_weight);
    }

    // -------------------------------------------------------------------------

    let currrent = indirect_diffuse_samples.read(screen_pos).xyz();

    let out = if previous.w == 0.0 {
        currrent.extend(1.0)
    } else {
        let previous = previous.xyz() / previous.w;
        let speed = 1.0 / (1.0 + history);

        previous
            .lerp(currrent, speed)
            .extend((history + 1.0).min(MAX_HISTORY))
    };

    unsafe {
        indirect_diffuse_colors.write(screen_pos, out);
    }
}
