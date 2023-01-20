#![no_std]

use spirv_std::glam::{
    vec2, vec3, UVec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};
use spirv_std::{spirv, Image, Sampler};
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
    #[spirv(descriptor_set = 0, binding = 2, storage_buffer)]
    lights: &[Light],
    #[spirv(descriptor_set = 0, binding = 3, storage_buffer)]
    materials: &[Material],
    #[spirv(descriptor_set = 0, binding = 4)]
    images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
    #[spirv(descriptor_set = 0, binding = 5)]
    samplers: &[Sampler; MAX_IMAGES],
    #[spirv(descriptor_set = 0, binding = 6, uniform)]
    world: &World,
    #[spirv(descriptor_set = 1, binding = 0, uniform)]
    camera: &Camera,
    #[spirv(descriptor_set = 1, binding = 1, storage_buffer)]
    ray_origins: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 2, storage_buffer)]
    ray_directions: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 3, storage_buffer)]
    ray_throughputs: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    ray_hits: &mut [Vec4],
    #[spirv(descriptor_set = 1, binding = 5)]
    colors: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 6)]
    normals: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 7)]
    bvh_heatmap: &Image!(2D, format = rgba8, sampled = false),
) {
    main_inner(
        global_id,
        local_idx,
        params,
        stack,
        triangles,
        bvh,
        lights,
        materials,
        images,
        samplers,
        world,
        camera,
        ray_origins,
        ray_directions,
        ray_throughputs,
        ray_hits,
        colors,
        normals,
        bvh_heatmap,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec3,
    local_idx: u32,
    params: &RayPassParams,
    stack: BvhTraversingStack,
    triangles: &[Triangle],
    bvh: &[Vec4],
    lights: &[Light],
    materials: &[Material],
    images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
    samplers: &[Sampler; MAX_IMAGES],
    world: &World,
    camera: &Camera,
    ray_origins: &mut [Vec4],
    ray_directions: &mut [Vec4],
    ray_throughputs: &mut [Vec4],
    ray_hits: &mut [Vec4],
    colors: &Image!(2D, format = rgba16f, sampled = false),
    normals: &Image!(2D, format = rgba16f, sampled = false),
    bvh_heatmaps: &Image!(2D, format = rgba8, sampled = false),
) {
    let image_xy = global_id.xy().as_ivec2();

    // If the world is empty, bail out early.
    //
    // It's not as much as optimization as a work-around for an empty BVH - by
    // having this below as an early check, we don't have to special-case BVH
    // later.
    if world.triangle_count == 0 {
        unsafe {
            colors.write(image_xy, camera.clear_color().extend(1.0));
        }

        return;
    }

    let triangles = TrianglesView::new(triangles);
    let bvh = BvhView::new(bvh);
    let lights = LightsView::new(lights);
    let materials = MaterialsView::new(materials);

    let global_idx =
        global_id.x + global_id.y * camera.viewport_size().as_uvec2().x;

    let global_idx = global_idx as usize;

    // ---

    let ray;
    let ray_throughput;

    if params.is_casting_primary_rays() {
        ray = camera.ray(vec2(global_id.x as f32, global_id.y as f32));
        ray_throughput = vec3(1.0, 1.0, 1.0);
    } else {
        let direction =
            unsafe { ray_directions.get_unchecked(global_idx) }.xyz();

        // A ray without a direction means that it's a dead-ray, i.e. there's
        // nothing to shade here
        if direction == Vec3::ZERO {
            return;
        }

        let origin = unsafe { ray_origins.get_unchecked(global_idx) }.xyz();

        ray = Ray::new(origin, direction);
        ray_throughput = ray_throughputs[global_idx].xyz();
    }

    let hit = {
        let d0 = unsafe { *ray_hits.get_unchecked(2 * global_idx) };
        let d1 = unsafe { *ray_hits.get_unchecked(2 * global_idx + 1) };

        Hit::deserialize([d0, d1], ray)
    };

    // ---

    let color;
    let normal;
    let next_ray_origin;
    let next_ray_direction;
    let next_ray_throughput;

    if hit.is_some() {
        let material = materials.get(MaterialId::new(hit.material_id));
        let mut noise = Noise::new(params.seed, global_idx);

        let (albedo, shade) = material.shade(
            local_idx, triangles, bvh, lights, world, images, samplers, stack,
            ray, hit, &mut noise,
        );

        color = shade * ray_throughput;
        normal = hit.normal.extend(f32::from_bits(hit.material_id));

        next_ray_origin = hit.point;

        next_ray_direction =
            (hit.normal + noise.sample_hemisphere()).normalize();

        next_ray_throughput = albedo * ray_throughput;
    } else {
        color = camera.clear_color();
        normal = Vec4::ZERO;
        next_ray_origin = Vec3::ZERO;
        next_ray_direction = Vec3::ZERO;
        next_ray_throughput = Vec3::ZERO;
    }

    unsafe {
        if params.bounce == 0 {
            let color = {
                let prev_normal: Vec4 = normals.read(image_xy);

                // Now we're going to apply denoising - it goes like this:
                //
                // - if the previous pixel was a hit
                //
                // - and the previously-hit normal is similar to the normal we
                //   hit now (i.e. it's probably a similar object),
                //
                // - and the previously-hit material id is the same as the
                //   material we hit now,
                //
                // - then blend the previous pixel with the current one.
                //
                // TODO implement ReSTIR GI instead
                if params.apply_denoising()
                    && prev_normal.xyz() != Vec3::ZERO
                    && prev_normal.xyz().distance_squared(normal.xyz()) < 0.01
                    && prev_normal.w == normal.w
                {
                    let prev_color: Vec4 = colors.read::<_, Vec4, 4>(image_xy);

                    ((prev_color.xyz() / prev_color.w) * 10.0 + color)
                        .extend(11.0)
                } else {
                    color.extend(1.0)
                }
            };

            let bvh_heatmap =
                Vec3::splat(hit.traversed_nodes as f32 / 100.0).extend(1.0);

            colors.write(image_xy, color);
            normals.write(image_xy, normal);
            bvh_heatmaps.write(image_xy, bvh_heatmap);
        } else if hit.is_some() {
            colors.write(
                image_xy,
                colors.read::<_, Vec4, 4>(image_xy) + color.extend(1.0),
            );
        }

        *ray_origins.get_unchecked_mut(global_idx) =
            next_ray_origin.extend(0.0);

        *ray_directions.get_unchecked_mut(global_idx) =
            next_ray_direction.extend(0.0);

        *ray_throughputs.get_unchecked_mut(global_idx) =
            next_ray_throughput.extend(0.0);
    }
}
