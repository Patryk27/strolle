#![no_std]

use spirv_std::glam::{vec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_models::{Camera, RasterPassParams};

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[spirv(vertex)]
pub fn main_vs(
    // Params
    #[spirv(descriptor_set = 0, binding = 0, uniform)]
    camera: &Camera,

    // Inputs
    in_d0: Vec4,
    in_d1: Vec4,
    in_d2: Vec4,

    // Outputs
    #[spirv(position)]
    out_pos: &mut Vec4,
    out_hit_point: &mut Vec3,
    out_hit_normal: &mut Vec3,
    #[spirv(flat)]
    out_hit_flat_normal: &mut Vec3,
    #[spirv(flat)]
    out_hit_tangent: &mut Vec4,
    out_hit_uv: &mut Vec2,
) {
    let position = in_d0.xyz();
    let normal = in_d1.xyz();
    let uv = vec2(in_d0.w, in_d1.w);
    let tangent = in_d2;

    *out_pos = camera.project(position);
    *out_hit_point = position;
    *out_hit_normal = normal;
    *out_hit_flat_normal = normal;
    *out_hit_tangent = tangent;
    *out_hit_uv = uv;
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[spirv(fragment)]
pub fn main_fs(
    // Params
    #[spirv(push_constant)]
    params: &RasterPassParams,

    #[spirv(descriptor_set = 1, binding = 0)]
    _base_color_texture: &Image!(2D, type = f32, sampled),
    #[spirv(descriptor_set = 1, binding = 1)]
    _base_color_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 2)]
    normal_map_texture: &Image!(2D, type = f32, sampled),
    #[spirv(descriptor_set = 1, binding = 3)]
    normal_map_sampler: &Sampler,

    // Inputs
    in_hit_point: Vec3,
    in_hit_normal: Vec3,
    #[spirv(flat)]
    in_hit_flat_normal: Vec3,
    #[spirv(flat)]
    in_hit_tangent: Vec4,
    in_hit_uv: Vec2,

    // Outputs
    out_hit_d0: &mut Vec4,
    out_hit_d1: &mut Vec4,
    out_hit_d2: &mut Vec4,
) {
    // If the model we're rendering uses per-vertex normals, the normal here
    // will be unnormalized due to the GPU interpolating it in-between vertices;
    // no biggie, let's just fix it:
    let hit_normal = in_hit_normal.normalize();

    let hit_normal = if params.has_normal_map == 1 {
        let tangent = in_hit_tangent.xyz();
        let bitangent = in_hit_tangent.w * hit_normal.cross(tangent);

        let mapped_normal = normal_map_texture.sample(*normal_map_sampler, in_hit_uv);
        let mapped_normal = 2.0 * mapped_normal - 1.0;

        (mapped_normal.x * tangent
            + mapped_normal.y * bitangent
            + mapped_normal.z * hit_normal).normalize()
    } else {
        hit_normal
    };

    *out_hit_d0 = in_hit_point.extend(f32::from_bits(params.material_id));
    *out_hit_d1 = hit_normal.extend(in_hit_uv.x);
    *out_hit_d2 = in_hit_flat_normal.extend(in_hit_uv.y);
}
