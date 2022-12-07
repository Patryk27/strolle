#![no_std]

use spirv_std::glam::{vec2, vec3, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_shader_common::*;

#[spirv(vertex)]
pub fn vs_main(
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

#[allow(clippy::too_many_arguments)]
#[spirv(fragment)]
pub fn fs_main(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)]
    static_geo: &StaticGeometry,
    #[spirv(uniform, descriptor_set = 1, binding = 0)]
    static_geo_index: &StaticGeometryIndex,
    #[spirv(uniform, descriptor_set = 1, binding = 1)]
    dynamic_geo: &DynamicGeometry,
    #[spirv(uniform, descriptor_set = 1, binding = 2)] uvs: &TriangleUvs,
    #[spirv(uniform, descriptor_set = 2, binding = 0)] camera: &Camera,
    #[spirv(uniform, descriptor_set = 2, binding = 1)] lights: &Lights,
    #[spirv(uniform, descriptor_set = 2, binding = 2)] materials: &Materials,
    #[spirv(descriptor_set = 3, binding = 0)] atlas_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 3, binding = 1)] atlas_sampler: &Sampler,
    color: &mut Vec4,
) {
    let world = World {
        static_geo,
        static_geo_index,
        dynamic_geo,
        uvs,
        camera,
        lights,
        materials,
        atlas_tex,
        atlas_sampler,
    };

    camera.ray(pos.xy()).shade(color, &world);

    *color = deband(pos.xy(), *color);
}

fn deband(pos: Vec2, color: Vec4) -> Vec4 {
    /// Thanks to https://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf (slide 49)
    fn screen_space_dither(pos: Vec2) -> Vec3 {
        let dither = Vec3::splat(vec2(171.0, 231.0).dot(pos));
        let dither = (dither / vec3(103.0, 71.0, 97.0)).fract();

        (dither - 0.5) / 255.0
    }

    let color = color.xyz().powf(1.0 / 2.2);
    let color = color + screen_space_dither(pos);
    let color = color.powf(2.2);

    color.extend(1.0)
}
