#![no_std]

use spirv_std::glam::{UVec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;
use strolle_models::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(local_invocation_index)] local_idx: u32,
    #[spirv(workgroup)] stack: BvhTraversingStack,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)]
    triangles: &[Triangle],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    instances: &[Instance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] bvh: &[Vec4],
    #[spirv(uniform, descriptor_set = 0, binding = 3)] lights: &[Light],
    #[spirv(uniform, descriptor_set = 0, binding = 4)] materials: &[Material],
    #[spirv(uniform, descriptor_set = 0, binding = 7)] info: &Info,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Camera,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)]
    rays: &mut [Vec4],
) {
    // If the world is empty, bail out early.
    //
    // It's not as much as optimization as a work-around for an empty BVH - by
    // having this below as an early check, we don't have to special-case BVH
    // later.
    if info.is_world_empty() {
        return;
    }

    let global_idx = id.y * camera.viewport_size().as_uvec2().x + id.x;
    let ray_idx = 2 * (global_idx as usize);
    let ray_d0 = unsafe { *rays.get_unchecked(ray_idx) };
    let ray_d1 = unsafe { *rays.get_unchecked(ray_idx + 1) };
    let ray_mode = ray_d0.w.to_bits();

    if ray_mode == 0 {
        return;
    }

    let world = World {
        global_idx,
        local_idx,
        triangles: TrianglesView::new(triangles),
        instances: InstancesView::new(instances),
        bvh: BvhView::new(bvh),
        lights: LightsView::new(lights),
        materials: MaterialsView::new(materials),
        info,
    };

    let (instance_id, triangle_id) =
        Ray::new(ray_d0.xyz(), ray_d1.xyz()).trace(&world, stack);

    unsafe {
        *rays.get_unchecked_mut(ray_idx) = ray_d0
            .xyz()
            .extend(f32::from_bits((instance_id << 2) | ray_mode));

        *rays.get_unchecked_mut(ray_idx + 1) =
            ray_d1.xyz().extend(f32::from_bits(triangle_id));
    }
}
