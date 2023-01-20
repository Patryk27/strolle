#![no_std]

use spirv_std::glam::{vec2, UVec3, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
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
    params: &RayPassParams,
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
    #[spirv(descriptor_set = 1, binding = 1, storage_buffer)]
    ray_origins: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 2, storage_buffer)]
    ray_directions: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    ray_hits: &mut [Vec4],
) {
    main_inner(
        global_id,
        local_idx,
        params,
        triangles,
        bvh,
        world,
        stack,
        camera,
        ray_origins,
        ray_directions,
        ray_hits,
    );
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec3,
    local_idx: u32,
    params: &RayPassParams,
    triangles: &[Triangle],
    bvh: &[Vec4],
    world: &World,
    stack: BvhTraversingStack,
    camera: &Camera,
    ray_origins: &mut [Vec4],
    ray_directions: &mut [Vec4],
    ray_hits: &mut [Vec4],
) {
    // If the world is empty, bail out early.
    //
    // It's not as much as optimization as a work-around for an empty BVH - by
    // having this below as an early check, we don't have to special-case BVH
    // later.
    if world.triangle_count == 0 {
        return;
    }

    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);

    let global_idx =
        global_id.x + global_id.y * camera.viewport_size().as_uvec2().x;

    let global_idx = global_idx as usize;

    // ---

    let ray = if params.is_casting_primary_rays() {
        camera.ray(vec2(global_id.x as f32, global_id.y as f32))
    } else {
        let direction =
            unsafe { *ray_directions.get_unchecked(global_idx) }.xyz();

        // A ray without a direction means that it's a dead-ray, i.e. there's
        // nothing to trace here
        if direction == Vec3::ZERO {
            return;
        }

        let origin = unsafe { *ray_origins.get_unchecked(global_idx) }.xyz();

        Ray::new(origin, direction)
    };

    let [hit_d0, hit_d1] = ray
        .trace_nearest(local_idx, triangles, bvh, stack)
        .serialize();

    ray_hits[2 * global_idx] = hit_d0;
    ray_hits[2 * global_idx + 1] = hit_d1;
}
