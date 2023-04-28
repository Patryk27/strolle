#![no_std]

use spirv_std::glam::{uvec2, UVec2, UVec3, Vec3Swizzles, Vec4};
use spirv_std::{spirv, Image};
use strolle_models::*;

#[rustfmt::skip]
#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main(
    #[spirv(global_invocation_id)]
    global_id: UVec3,
    #[spirv(local_invocation_index)]
    local_idx: u32,
    #[spirv(push_constant)]
    params: &VoxelShadingPassParams,
    #[spirv(workgroup)]
    stack: BvhTraversingStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    primary_hits_d0: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 2)]
    primary_hits_d1: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 3)]
    primary_hits_d2: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    pending_voxels: &mut [Vec4],
) {
    main_inner(
        global_id.xy(),
        local_idx,
        params,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        camera,
        primary_hits_d0,
        primary_hits_d1,
        primary_hits_d2,
        PendingVoxelHitsViewMut::new(pending_voxels),
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    local_idx: u32,
    params: &VoxelShadingPassParams,
    stack: BvhTraversingStack,
    triangles: TrianglesView,
    bvh: BvhView,
    camera: &Camera,
    primary_hits_d0: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d1: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d2: &Image!(2D, format = rgba32f, sampled = false),
    mut pending_voxel_hits: PendingVoxelHitsViewMut,
) {
    let viewport_width = camera.viewport_size().x;
    let pending_voxels_width = viewport_width / 2;
    let global_idx = global_id.x + global_id.y * pending_voxels_width;

    let mut noise = Noise::new(params.seed, global_id.x, global_id.y);

    let primary_hit = {
        let sample = noise.sample_int();
        let delta_x = sample & 0b11;
        let delta_y = (sample >> 2) & 0b11;
        let image_xy = 2 * global_id + uvec2(delta_x, delta_y);
        let ray = camera.ray(image_xy);

        Hit::from_primary(
            primary_hits_d0.read(image_xy),
            primary_hits_d1.read(image_xy),
            primary_hits_d2.read(image_xy),
            ray,
        )
    };

    let hit = if primary_hit.is_none() {
        Hit::none()
    } else {
        let ray = Ray::new(
            primary_hit.point,
            noise.sample_hemisphere(primary_hit.normal),
        );

        ray.trace_nearest(local_idx, triangles, bvh, stack).0
    };

    pending_voxel_hits.set(
        PendingVoxelId::new(global_idx),
        PendingVoxelHit::from_hit(hit),
    );
}
