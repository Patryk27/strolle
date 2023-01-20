#![no_std]

use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_models::*;

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
    #[spirv(push_constant)] params: &DrawingPassParams,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 0, binding = 1)] colors_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)] colors_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] normals_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 4)] normals_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 5)] bvh_heatmap_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 6)] bvh_heatmap_sampler: &Sampler,
    frag_color: &mut Vec4,
) {
    let texel_xy = {
        let viewport_pos = camera.viewport_pos();
        let viewport_size = camera.viewport_size();

        let x = (pos.x - viewport_pos.x) / (viewport_size.x);
        let y = (pos.y - viewport_pos.y) / (viewport_size.y);

        vec2(x, y)
    };

    *frag_color = match params.viewport_mode {
        0 => {
            let color: Vec4 = colors_tex.sample(*colors_sampler, texel_xy);
            let color = color.xyz() / color.w;

            deband(pos.xy(), color).extend(1.0)
        }

        1 => {
            let normal: Vec4 = normals_tex.sample(*normals_sampler, texel_xy);

            normal.xyz().extend(1.0)
        }

        2 => {
            let color: Vec4 =
                bvh_heatmap_tex.sample(*bvh_heatmap_sampler, texel_xy);

            color.xyz().extend(1.0)
        }

        _ => vec4(0.0, 0.0, 0.0, 1.0),
    };
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
