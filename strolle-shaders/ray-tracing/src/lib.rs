#![no_std]

use spirv_std::glam::{UVec2, UVec3, Vec3, Vec3Swizzles, Vec4};
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
    #[spirv(workgroup)]
    stack: BvhTraversingStack,
    #[spirv(descriptor_set = 0, binding = 0, storage_buffer)]
    triangles: &[Triangle],
    #[spirv(descriptor_set = 0, binding = 1, storage_buffer)]
    bvh: &[Vec4],
    #[spirv(descriptor_set = 0, binding = 2, uniform)]
    world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1)]
    directs: &Image!(2D, format = rgba16f, sampled = false),
) {
    main_inner(
        global_id.xy(),
        local_idx,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        world,
        camera,
        directs,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    local_idx: u32,
    stack: BvhTraversingStack,
    triangles: TrianglesView,
    bvh: BvhView,
    _world: &World,
    camera: &Camera,
    directs: &Image!(2D, format = rgba16f, sampled = false),
) {
    let (_, traversed_nodes) = camera
        .ray(global_id)
        .trace_nearest(local_idx, triangles, bvh, stack);

    unsafe {
        directs.write(
            global_id,
            Vec3::splat(traversed_nodes as f32 / 200.0).extend(1.0),
        );
    }
}
