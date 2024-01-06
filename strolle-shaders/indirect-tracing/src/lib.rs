#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(push_constant)] params: &IndirectPassParams,
    #[spirv(workgroup)] stack: BvhStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)] blue_noise_sobol: &[u32],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    blue_noise_scrambling_tile: &[u32],
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    blue_noise_ranking_tile: &[u32],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 4, storage_buffer)] bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 5, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 6)] atlas_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 7)] atlas_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 0, uniform)] camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)] bevy_gbuffer_sampler: &Sampler,
    #[spirv(descriptor_set = 1, binding = 2)] bevy_gbuffer_normals: Tex,
    #[spirv(descriptor_set = 1, binding = 3)] bevy_gbuffer_depth: Tex,
    #[spirv(descriptor_set = 1, binding = 4)] indirect_rays: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 5)] indirect_gbuffer_d0: TexRgba32,
    #[spirv(descriptor_set = 1, binding = 6)] indirect_gbuffer_d1: TexRgba32,
) {
    let screen_pos = global_id.xy();
    let mut bnoise = LdsBlueNoise::new(
        blue_noise_sobol,
        blue_noise_scrambling_tile,
        blue_noise_ranking_tile,
        screen_pos,
        params.frame,
        0,
    );
    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let materials = MaterialsView::new(materials);

    if !camera.contains(screen_pos) {
        return;
    }

    // -------------------------------------------------------------------------

    let screen_uv = screen_pos.as_vec2() / camera.screen_size().as_vec2();

    let direct_hit_depth = bevy_gbuffer_depth
        .sample_by_lod(*bevy_gbuffer_sampler, screen_uv, 0.0)
        .x;

    let direct_hit_point = camera.ray(screen_pos).at(direct_hit_depth);

    let direct_hit_normal = bevy_gbuffer_normals
        .sample_by_lod(*bevy_gbuffer_sampler, screen_uv, 0.0)
        .xyz();

    if direct_hit_depth <= 0.0 {
        unsafe {
            indirect_rays.write(screen_pos, Vec4::ZERO);
        }

        return;
    }

    // ---

    let indirect_ray_direction = bnoise.sample_hemisphere(direct_hit_normal);

    let ray = Ray::new(
        direct_hit_point + direct_hit_normal * 0.001,
        indirect_ray_direction,
    );

    let (indirect_hit, _) = ray.trace(
        local_idx,
        stack,
        triangles,
        bvh,
        materials,
        atlas_tex,
        atlas_sampler,
    );

    // ---

    let indirect_gbuffer = if indirect_hit.is_some() {
        let mut indirect_material = materials.get(indirect_hit.material_id);

        indirect_material.adjust_for_indirect();

        GBufferEntry {
            base_color: indirect_material.base_color(
                atlas_tex,
                atlas_sampler,
                indirect_hit.uv,
            ),
            normal: indirect_hit.normal,
            metallic: indirect_material.metallic,
            emissive: indirect_material.emissive(
                atlas_tex,
                atlas_sampler,
                indirect_hit.uv,
            ),
            roughness: indirect_material.roughness,
            reflectance: indirect_material.reflectance,
            depth: direct_hit_point.distance(indirect_hit.point),
        }
    } else {
        Default::default()
    };

    let [d0, d1] = indirect_gbuffer.pack();

    unsafe {
        indirect_rays
            .write(screen_pos, indirect_ray_direction.extend(direct_hit_depth));

        indirect_gbuffer_d0.write(screen_pos, d0);
        indirect_gbuffer_d1.write(screen_pos, d1);
    }
}
