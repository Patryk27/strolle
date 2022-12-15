#![no_std]

use spirv_std::glam::{vec2, vec3, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_renderer_models::*;

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
pub fn main_fs(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] params: &Params,
    #[spirv(descriptor_set = 0, binding = 1)] image_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] image_sampler: &Sampler,
    color: &mut Vec4,
) {
    let texel_xy = {
        let x = (pos.x - params.x) / (params.w);
        let y = (pos.y - params.y) / (params.h);

        vec2(x, y)
    };

    let texel: Vec4 = image_tex.sample(*image_sampler, texel_xy);

    *color = deband(pos.xy(), texel.xyz()).extend(1.0);
}

fn deband(pos: Vec2, color: Vec3) -> Vec3 {
    /// Thanks to https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf (slide 49)
    fn screen_space_dither(pos: Vec2) -> Vec3 {
        let dither = Vec3::splat(vec2(171.0, 231.0).dot(pos));
        let dither = (dither / vec3(103.0, 71.0, 97.0)).fract();

        (dither - 0.5) / 255.0
    }

    let color = color.powf(1.0 / 2.2);
    let color = color + screen_space_dither(pos);

    color.powf(2.2)
}
