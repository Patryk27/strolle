#![no_std]

use spirv_std::glam::{vec2, UVec3, Vec4};
use spirv_std::spirv;
use strolle_models::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera: &Camera,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    rays: &mut [Vec4],
) {
    let global_idx = id.y * camera.viewport_size().as_uvec2().x + id.x;
    let ray = camera.ray(vec2(id.x as f32, id.y as f32));
    let ray_idx = 2 * (global_idx as usize);
    let ray_mode = 1;

    unsafe {
        *rays.get_unchecked_mut(ray_idx) =
            ray.origin().extend(f32::from_bits(ray_mode));

        *rays.get_unchecked_mut(ray_idx + 1) =
            ray.direction().extend(f32::from_bits(0));
    }
}
