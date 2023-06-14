#![no_std]

use spirv_std::glam::{vec2, vec3, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_gpu::*;

#[spirv(vertex)]
pub fn main_vs(
    #[spirv(vertex_index)] vert_idx: i32,
    #[spirv(position)] output: &mut Vec4,
) {
    fn full_screen_triangle(vert_idx: i32) -> Vec4 {
        let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
        let pos = 2.0 * uv - Vec2::ONE;

        pos.extend(0.0).extend(1.0)
    }

    *output = full_screen_triangle(vert_idx);
}

#[spirv(fragment)]
#[allow(clippy::too_many_arguments)]
pub fn main_fs(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(push_constant)] params: &OutputDrawingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] direct_colors_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] direct_colors_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] direct_hits_d2_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 4)] direct_hits_d2_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 5)] indirect_colors_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 6)] indirect_colors_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 7)] geometry_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 8)] geometry_sampler: &Sampler,
    frag_color: &mut Vec4,
) {
    let texel_xy = {
        let viewport_pos = camera.viewport_position();
        let viewport_size = camera.viewport_size().as_vec2();

        let x = (pos.x - viewport_pos.x) / (viewport_size.x);
        let y = (pos.y - viewport_pos.y) / (viewport_size.y);

        vec2(x, y)
    };

    let (color, apply_color_adjustments) = match params.viewport_mode {
        0 => {
            let albedo = direct_hits_d2_tex
                .sample(*direct_hits_d2_sampler, texel_xy)
                .xyz();

            let direct = direct_colors_tex
                .sample(*direct_colors_sampler, texel_xy)
                .xyz();

            let indirect = indirect_colors_tex
                .sample(*indirect_colors_sampler, texel_xy)
                .xyz();

            ((direct + albedo * indirect), true)
        }

        1 => {
            let direct = direct_colors_tex
                .sample(*direct_colors_sampler, texel_xy)
                .xyz();

            (direct, true)
        }

        2 => {
            let indirect = indirect_colors_tex
                .sample(*indirect_colors_sampler, texel_xy)
                .xyz();

            (indirect, true)
        }

        3 => {
            let normal = geometry_tex.sample(*geometry_sampler, texel_xy).xyz();

            (normal, false)
        }

        4 => {
            let heatmap = direct_colors_tex
                .sample(*direct_colors_sampler, texel_xy)
                .xyz();

            (heatmap, false)
        }

        _ => Default::default(),
    };

    *frag_color = if apply_color_adjustments {
        let color = apply_tone_mapping(color);
        let color = apply_debanding(pos.xy(), color);

        color.extend(1.0)
    } else {
        color.extend(1.0)
    };
}

/// Applies ACES tone mapping.
///
/// Thanks to:
/// https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
fn apply_tone_mapping(x: Vec3) -> Vec3 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;

    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(Vec3::ZERO, Vec3::ONE)
}

/// Applies screen-space debanding using a simple dither.
///
/// Thanks to:
/// https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf (slide 49)
fn apply_debanding(pos: Vec2, color: Vec3) -> Vec3 {
    fn screen_space_dither(pos: Vec2) -> Vec3 {
        let dither = Vec3::splat(vec2(171.0, 231.0).dot(pos));
        let dither = (dither / vec3(103.0, 71.0, 97.0)).fract();

        (dither - 0.5) / 255.0
    }

    let color = color.powf(1.0 / 2.2);
    let color = color + screen_space_dither(pos);

    color.powf(2.2)
}
