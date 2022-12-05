#![no_std]

use spirv_std::glam::{vec2, Vec2, Vec4, Vec4Swizzles};
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
}
