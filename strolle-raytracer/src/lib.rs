#![no_std]

use spirv_std::glam::{vec2, UVec3, Vec4};
use spirv_std::spirv;
use strolle_raytracer_models::*;

#[allow(clippy::too_many_arguments)]
#[spirv(compute(threads(8, 8)))]
pub fn main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)]
    geometry_tris: &[Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)]
    geometry_uvs: &[Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)]
    geometry_bvh: &[Vec4],
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Camera,
    #[spirv(uniform, descriptor_set = 1, binding = 1)] lights: &Lights,
    #[spirv(uniform, descriptor_set = 1, binding = 2)] materials: &Materials,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 0)]
    image: &mut [f32],
) {
    let world = World {
        geometry_tris: GeometryTrisView::new(geometry_tris),
        geometry_uvs: GeometryUvsView::new(geometry_uvs),
        geometry_bvh: GeometryBvhView::new(geometry_bvh),
        camera,
        lights,
        materials,
    };

    let color = world
        .camera
        .ray(vec2(id.x as f32, id.y as f32))
        .shade(&world);

    let idx = ((id.x + id.y * camera.viewport_size().x) * 3) as usize;

    image[idx] = color.x;
    image[idx + 1] = color.y;
    image[idx + 2] = color.z;
}
