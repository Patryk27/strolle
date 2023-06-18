#![no_std]

use spirv_std::glam::{vec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
use strolle_gpu::{
    Camera, DirectRasterPassParams, Material, MaterialId, MaterialsView, Normal,
};

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[spirv(vertex)]
pub fn main_vs(
    // Params
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,

    // Inputs
    vertex_d0: Vec4,
    vertex_d1: Vec4,
    _vertex_d2: Vec4,

    // Outputs
    #[spirv(position)]
    out_pos: &mut Vec4,
    out_hit_point: &mut Vec3,
    out_hit_normal: &mut Vec3,
    // #[spirv(flat)]
    // out_hit_tangent: &mut Vec4,
    out_hit_uv: &mut Vec2,
) {
    let position = vertex_d0.xyz();
    let normal = vertex_d1.xyz();
    let uv = vec2(vertex_d0.w, vertex_d1.w);
    // let tangent = vertex_d2;

    *out_pos = camera.world_to_clip(position);
    *out_hit_point = position;
    *out_hit_normal = normal;
    // *out_hit_tangent = tangent;
    *out_hit_uv = uv;
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[spirv(fragment)]
pub fn main_fs(
    // Params
    #[spirv(push_constant)]
    params: &DirectRasterPassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 1)]
    atlas_tex: &Image!(2D, type=f32, sampled),
    #[spirv(descriptor_set = 0, binding = 2)]
    atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,

    // Inputs
    hit_point: Vec3,
    hit_normal: Vec3,
    // #[spirv(flat)]
    // hit_tangent: Vec4,
    hit_uv: Vec2,

    // Outputs
    out_direct_hits_d0: &mut Vec4,
    out_direct_hits_d1: &mut Vec4,
    out_direct_hits_d2: &mut Vec4,
    out_surface_map: &mut Vec4,
) {
    let material = MaterialsView::new(materials)
        .get(MaterialId::new(params.material_id));

    let hit_albedo = material.albedo(atlas_tex, atlas_sampler, hit_uv);

    let hit_normal = {
        // If the mesh we're rendering uses per-vertex normals, the normal here
        // will be unnormalized due to the GPU interpolating it in-between
        // vertices; no biggie, let's just fix it:
        hit_normal.normalize()
    };

    let hit_normal2 = Normal::encode(hit_normal);
    let hit_distance = camera.origin().distance(hit_point);

    *out_direct_hits_d0 = hit_point.extend(f32::from_bits(params.material_id));
    *out_direct_hits_d1 = hit_normal2.extend(hit_uv.x).extend(hit_uv.y);
    *out_direct_hits_d2 = hit_albedo;
    *out_surface_map = hit_normal.extend(hit_distance);
}
