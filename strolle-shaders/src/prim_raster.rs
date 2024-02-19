use spirv_std::arch;
use strolle_gpu::prelude::*;

#[allow(clippy::too_many_arguments)]
#[spirv(vertex)]
pub fn vs(
    // Params
    #[spirv(push_constant)] params: &PrimRasterPassParams,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,

    // Inputs
    vertex_d0: Vec4,
    vertex_d1: Vec4,

    // Outputs
    #[spirv(position)] out_vertex: &mut Vec4,
    out_curr_vertex: &mut Vec4,
    out_prev_vertex: &mut Vec4,
    out_point: &mut Vec3,
    out_normal: &mut Vec3,
    out_uv: &mut Vec2,
) {
    let point = vertex_d0.xyz();

    let prev_point = params
        .prev_xform()
        .transform_point3(params.curr_xform_inv().transform_point3(point));

    let normal = vertex_d1.xyz();
    let uv = vec2(vertex_d0.w, vertex_d1.w);

    *out_vertex = camera.world_to_clip(point);
    *out_curr_vertex = camera.world_to_clip(point);
    *out_prev_vertex = prev_camera.world_to_clip(prev_point);
    *out_point = point;
    *out_normal = normal;
    *out_uv = uv;
}

#[allow(clippy::too_many_arguments)]
#[spirv(fragment)]
pub fn fs(
    // Params
    #[spirv(push_constant)] params: &PrimRasterPassParams,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 1)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 2)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, uniform)] prev_camera: &Camera,
    #[spirv(front_facing)] front_facing: bool,

    // Inputs
    curr_vertex: Vec4,
    prev_vertex: Vec4,
    point: Vec3,
    normal: Vec3,
    uv: Vec2,

    // Outputs
    out_prim_gbuffer_d0: &mut Vec4,
    out_prim_gbuffer_d1: &mut Vec4,
    out_surface: &mut Vec4,
    out_velocity: &mut Vec4,
) {
    let material = MaterialsView::new(materials)
        .get(MaterialId::new(params.material_id()));

    let base_color = material.base_color(atlas_tex, atlas_sampler, uv);
    let metallic_roughness =
        material.metallic_roughness(atlas_tex, atlas_sampler, uv);
    // If our material is transparent and doesn't rely on refraction, kill the
    // current fragment to re-use GPU in finding the next triangle
    if base_color.w < 0.01 && material.ior == 1.0 {
        arch::kill();
    }

    let normal = {
        // TODO bring back normal mapping
        let normal = normal.normalize();

        if front_facing {
            normal
        } else {
            -normal
        }
    };

    let ray = camera.ray(camera.clip_to_screen(curr_vertex).round().as_uvec2());
    let depth = ray.origin().distance(point);

    let gbuffer = GBufferEntry {
        base_color,
        normal,
        metallic: metallic_roughness.x,
        emissive: material.emissive(atlas_tex, atlas_sampler, uv),
        roughness: metallic_roughness.y,
        reflectance: material.reflectance,
        depth,
    };

    let [gbuffer_d0, gbuffer_d1] = gbuffer.pack();

    *out_prim_gbuffer_d0 = gbuffer_d0;
    *out_prim_gbuffer_d1 = gbuffer_d1;

    // -------------------------------------------------------------------------

    *out_surface = Normal::encode(normal)
        .extend(depth)
        .extend(material.roughness);

    // -------------------------------------------------------------------------

    *out_velocity = {
        let velocity = camera.clip_to_screen(curr_vertex)
            - prev_camera.clip_to_screen(prev_vertex);

        if velocity.length_squared() >= 0.001 {
            velocity.extend(0.0).extend(0.0)
        } else {
            // Due to floting-point inaccuracies, stationary objects can end up
            // having a very small velocity instead of zero - this causes our
            // reprojection shader to freak out, so let's truncate small
            // velocities to zero
            Default::default()
        }
    };
}
