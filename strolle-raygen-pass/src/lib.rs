#![no_std]

use spirv_std::glam::{vec2, UVec3};
use spirv_std::spirv;
use strolle_models::*;

#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera: &Camera,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    rays: &mut [RayOp],
) {
    let global_idx =
        global_id.x + global_id.y * camera.viewport_size().as_uvec2().x;

    let ray = camera.ray(vec2(global_id.x as f32, global_id.y as f32));
    let ray = RayOp::primary(ray);

    RayOpsView::new(rays).set(global_idx, ray)
}
