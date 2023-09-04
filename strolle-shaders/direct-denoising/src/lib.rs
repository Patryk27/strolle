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

        previous = sample.xyz().extend(1.0);
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
        sample_radius += 1.0;
        sample_angle += GOLDEN_ANGLE;

        let sample_offset =
            vec2(sample_angle.sin(), sample_angle.cos()) * sample_radius;

        let sample_pos_flt = if reprojection.is_some() {
            reprojection.prev_pos() + sample_offset
        } else {
            screen_pos.as_vec2() + sample_offset
        };

        let sample_pos = sample_pos_flt.as_ivec2();
        let sample_pos = camera.contain(sample_pos);
        let sample_surface = prev_surface_map.get(sample_pos);

        let sample_reprojection = {
            let check_validity = move |sample_pos| {
                if !camera.contains(sample_pos) {
                    return false;
                }

                prev_surface_map
                    .get(sample_pos.as_uvec2())
                    .evaluate_similarity_to(&surface)
                    >= 0.9
            };

            let mut validity = 0;

            let [p00, p10, p01, p11] = BilinearFilter::reprojection_coords(
                sample_pos_flt.x,
                sample_pos_flt.y,
            );

            if check_validity(p00) {
                validity |= 0b0001;
            }

            if check_validity(p10) {
                validity |= 0b0010;
            }

            if check_validity(p01) {
                validity |= 0b0100;
            }

            if check_validity(p11) {
                validity |= 0b1000;
            }

            Reprojection {
                prev_x: sample_pos_flt.x,
                prev_y: sample_pos_flt.y,
                confidence: 1.0,
                validity,
            }
        };

        let sample_color =
            BilinearFilter::reproject(sample_reprojection, move |pos| {
                (prev_direct_colors.read(pos), 1.0)
            });

        let sample_weight = sample_surface.evaluate_similarity_to(&surface);

        previous += (sample_color.xyz() * sample_weight).extend(sample_weight);
    }

    // -------------------------------------------------------------------------

    let currrent = direct_samples.read(screen_pos).xyz();

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
        direct_colors.write(screen_pos, out);
    }
}
