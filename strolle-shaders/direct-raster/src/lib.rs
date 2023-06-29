#![no_std]

use strolle_gpu::prelude::*;

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[spirv(vertex)]
pub fn main_vs(
    // Params
    #[spirv(push_constant)]
    params: &DirectRasterPassParams,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)]
    prev_camera: &Camera,

    // Inputs
    vertex_d0: Vec4,
    vertex_d1: Vec4,

    // Outputs
    #[spirv(position)]
    out_vertex: &mut Vec4,
    out_curr_vertex: &mut Vec4,
    out_prev_vertex: &mut Vec4,
    out_hit_point: &mut Vec3,
    out_hit_normal: &mut Vec3,
    out_hit_uv: &mut Vec2,
) {
    let position = vertex_d0.xyz();

    let prev_position = params.prev_xform().transform_point3(
        params.curr_xform_inv().transform_point3(position)
    );

    let normal = vertex_d1.xyz();
    let uv = vec2(vertex_d0.w, vertex_d1.w);

    *out_vertex = camera.world_to_clip(position);
    *out_curr_vertex = camera.world_to_clip(position);
    *out_prev_vertex = prev_camera.world_to_clip(prev_position);
    *out_hit_point = position;
    *out_hit_normal = normal;
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
    #[spirv(descriptor_set = 1, binding = 1, uniform)]
    prev_camera: &Camera,

    // Inputs
    curr_vertex: Vec4,
    prev_vertex: Vec4,
    hit_point: Vec3,
    hit_normal: Vec3,
    hit_uv: Vec2,

    // Outputs
    out_direct_hits_d0: &mut Vec4,
    out_direct_hits_d1: &mut Vec4,
    out_direct_hits_d2: &mut Vec4,
    out_direct_hits_d3: &mut Vec4,
    out_surface_map: &mut Vec4,
    out_velocity_map: &mut Vec4,
) {
    let material = MaterialsView::new(materials)
        .get(MaterialId::new(params.material_id()));

    let hit_albedo = material.albedo(atlas_tex, atlas_sampler, hit_uv);
    let hit_emissive = material.emissive(atlas_tex, atlas_sampler, hit_uv);

    let hit_normal = {
        // If the mesh we're rendering uses per-vertex normals, the normal here
        // will be unnormalized due to the GPU interpolating it in-between
        // vertices; no biggie, let's just fix it:
        hit_normal.normalize()
    };

    let hit_normal_encoded = Normal::encode(hit_normal);

    *out_direct_hits_d0 = hit_point.extend(f32::from_bits(params.material_id()));
    *out_direct_hits_d1 = hit_normal_encoded.extend(hit_uv.x).extend(hit_uv.y);
    *out_direct_hits_d2 = hit_albedo;
    *out_direct_hits_d3 = hit_emissive;

    *out_surface_map = hit_normal_encoded
        .extend(curr_vertex.z)
        .extend(f32::from_bits(params.material_id()));

    *out_velocity_map = {
        let curr_scren_pos = camera.clip_to_screen(curr_vertex);
        let prev_screen_pos = prev_camera.clip_to_screen(prev_vertex);

        (curr_scren_pos - prev_screen_pos).extend(0.0).extend(0.0)
    };
}
