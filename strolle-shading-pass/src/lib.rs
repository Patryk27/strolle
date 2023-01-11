#![no_std]

use spirv_std::glam::{UVec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::{spirv, Image, Sampler};
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
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)]
    lights: &[Light],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 5)] images: &[Image!(2D, type=f32, sampled);
         256],
    #[spirv(descriptor_set = 0, binding = 6)] samplers: &[Sampler; 256],
    #[spirv(uniform, descriptor_set = 0, binding = 7)] info: &Info,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] camera: &Camera,
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)]
    rays: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 2)] image: &Image!(
        2D,
        format = rgba16f,
        sampled = false
    ),
) {
    if info.is_world_empty() {
        return;
    }

    let global_idx = id.y * camera.viewport_size().as_uvec2().x + id.x;

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

    let ray_idx = 2 * (global_idx as usize);
    let ray_d0 = unsafe { *rays.get_unchecked(ray_idx) };
    let ray_d1 = unsafe { *rays.get_unchecked(ray_idx + 1) };

    let ray_i0 = ray_d0.w.to_bits();
    let ray_i1 = ray_d1.w.to_bits();

    let instance_id = ray_i0 >> 2;
    let ray_mode = ray_i0 & 0b11;
    let triangle_id = ray_i1;

    if ray_mode == 0 {
        return;
    }

    // ---

    let (color, continued_ray, continued_ray_mode) = if debug::ENABLE_AABB {
        // There's always at least one node traversed (the root node), so
        // subtracting one allows us to get pure black color for the background
        // if there's a miss
        let traversed_nodes = instance_id - 1;

        let color =
            spirv_std::glam::Vec3::splat((traversed_nodes as f32) / 200.0)
                .extend(1.0);

        (color, Default::default(), 0)
    } else {
        #[allow(clippy::collapsible_else_if)]
        if instance_id == Ray::MAX_INSTANCE_ID {
            let color = camera.clear_color().extend(1.0);

            (color, Default::default(), 0)
        } else {
            let instance_id = InstanceId::new(instance_id);
            let triangle_id = TriangleId::new(triangle_id);

            let instance = world.instances.get(instance_id);
            let ray = Ray::new(ray_d0.xyz(), ray_d1.xyz());

            // Load the triangle, convert it from mesh-space into world-space
            // and perform hit-testing.
            //
            // (we know this calculation must return `hit.is_some()`, because
            // otherwise the tracing-pass would have already returned `miss` and
            // we wouldn't get inside this conditional -- so in here we kinda
            // re-do the same computation as in the tracing-pass to avoid having
            // an extra huge buffer for storring hit-data in-between passes.)
            //
            // Note that during tracing, we convert *ray* from world-space into
            // mesh-space, but in here it's simpler to convert the *triangle*
            // from mesh-space into world-space, since it gives us hit-point and
            // hit-normal that are already in world-space.
            //
            // (otherwise we'd have to convert ray, lights and hit-data into
            // mesh-space, which requires more work.)
            let hit = world
                .triangles
                .get(triangle_id)
                .with_transform(instance.transform())
                .hit(ray);

            // Having the hit-data, load the material and compute lightning
            world
                .materials
                .get(instance.material_id())
                .shade(&world, images, samplers, stack, ray, hit)
        }
    };

    // ---

    unsafe {
        *rays.get_unchecked_mut(ray_idx) = continued_ray
            .origin()
            .extend(f32::from_bits(continued_ray_mode));

        *rays.get_unchecked_mut(ray_idx + 1) =
            continued_ray.direction().extend(f32::from_bits(0));
    }

    // ---

    let image_xy = id.xy().as_ivec2();

    let color = match ray_mode {
        // Primary ray
        1 => color,

        // Transparent ray
        2 => {
            let primary_color: Vec4 = image.read(image_xy);

            let color = primary_color.xyz() * primary_color.w
                + color.xyz() * (1.0 - primary_color.w);

            color.extend(1.0)
        }

        // Unreachable
        _ => color,
    };

    // TODO safety
    unsafe {
        image.write(image_xy, color);
    }
}
