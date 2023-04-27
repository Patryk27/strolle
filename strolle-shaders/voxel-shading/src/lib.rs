#![no_std]

use core::f32::consts::PI;

use spirv_std::glam::{
    uvec2, UVec2, UVec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::float::Float;
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
    params: &VoxelShadingPassParams,
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
    #[spirv(descriptor_set = 1, binding = 1)]
    primary_hits_d0: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 2)]
    primary_hits_d1: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 3)]
    primary_hits_d2: &Image!(2D, format = rgba32f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 4, storage_buffer)]
    voxels: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 5, storage_buffer)]
    pending_voxels: &mut [Vec4],
) {
    main_inner(
        global_id.xy(),
        local_idx,
        params,
        stack,
        TrianglesView::new(triangles),
        BvhView::new(bvh),
        LightsView::new(lights),
        MaterialsView::new(materials),
        images,
        samplers,
        world,
        camera,
        primary_hits_d0,
        primary_hits_d1,
        primary_hits_d2,
        voxels,
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
    lights: LightsView,
    materials: MaterialsView,
    _images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
    _samplers: &[Sampler; MAX_IMAGES],
    world: &World,
    camera: &Camera,
    primary_hits_d0: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d1: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d2: &Image!(2D, format = rgba32f, sampled = false),
    voxels: &[Vec4],
    pending_voxel_hits: PendingVoxelHitsViewMut,
) {
    let viewport_width = camera.viewport_size().x;
    let pending_voxels_width = viewport_width / 2;
    let global_idx = global_id.x + global_id.y * pending_voxels_width;
    let mut noise = Noise::new(params.seed, global_id.x, global_id.y);

    // TODO wouldn't it be easier to just store the ray.origin & ray.direction
    //      into the pending-voxel?
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

    let ray = Ray::new(
        primary_hit.point,
        noise.sample_hemisphere(primary_hit.normal),
    );

    let hit = pending_voxel_hits
        .get(PendingVoxelId::new(global_idx))
        .as_hit();

    let direct = if hit.is_some() {
        let material = materials.get(MaterialId::new(hit.material_id));
        // let albedo = material.albedo(images, samplers, hit).xyz();
        let albedo = material.base_color.xyz(); // TODO

        let mut direct = Vec3::ZERO;
        let mut light_id = 0;

        while light_id < world.light_count {
            let light = lights.get(LightId::new(light_id));

            direct += compute_direct_lightning_for(
                local_idx, triangles, bvh, stack, material, hit, ray, albedo,
                light,
            );

            light_id += 1;
        }

        direct
    } else {
        camera.clear_color()
    };

    // TODO this is incorrect because we might be looking at stale voxels
    let indirect = {
        let voxel = VoxelsView::new(voxels)
            .get(world.voxelize(hit.point, hit.flat_normal));

        if voxel.samples > 0.0 && voxel.color().length_squared() > 0.0 {
            voxel.color()
        } else {
            Default::default()
        }
    };

    let color =
        (direct + indirect) * ray.direction().dot(primary_hit.flat_normal);

    let voxel_id = world.voxelize(primary_hit.point, primary_hit.flat_normal);

    pending_voxel_hits.as_pending_voxels_view_mut().set(
        PendingVoxelId::new(global_idx),
        PendingVoxel {
            color,
            frame: params.frame,
            point: primary_hit.point,
            voxel_id,
        },
    );
}

/// Computes direct contribution of given light using a simplified model that
/// doesn't care about specular highlights or light radiuses.
///
/// (that's mostly because they are kinda costly to compute and they don't
/// contribute that much to *indirect* lightning to be worth it.)
#[allow(clippy::too_many_arguments)]
fn compute_direct_lightning_for(
    local_idx: u32,
    triangles: TrianglesView,
    bvh: BvhView,
    stack: BvhTraversingStack,
    material: Material,
    hit: Hit,
    ray: Ray,
    albedo: Vec3,
    light: Light,
) -> Vec3 {
    let is_occluded = {
        let light_pos = light.center();
        let light_to_hit = hit.point - light_pos;

        let shadow_ray = Ray::new(light_pos, light_to_hit.normalize());
        let max_distance = light_to_hit.length();

        shadow_ray.trace_any(local_idx, triangles, bvh, stack, max_distance)
    };

    if is_occluded {
        return Vec3::ZERO;
    }

    let roughness =
        perceptual_roughness_to_roughness(material.perceptual_roughness);

    let hit_to_light = light.center() - hit.point;
    let diffuse_color = albedo * (1.0 - material.metallic);
    let v = -ray.direction();
    let r = reflect(-v, hit.normal);

    let range = light.range();

    let l = hit_to_light.normalize();
    let n_o_l = saturate(hit.normal.dot(l));

    let diffuse = diffuse_light(l, v, hit, roughness, n_o_l);
    let center_to_ray = hit_to_light.dot(r) * r - hit_to_light;

    let closest_point = hit_to_light
        + center_to_ray
            * saturate(
                light.radius() * inverse_sqrt(center_to_ray.dot(center_to_ray)),
            );

    let l_spec_length_inverse = inverse_sqrt(closest_point.dot(closest_point));

    let l = closest_point * l_spec_length_inverse;
    let n_o_l = saturate(hit.normal.dot(l));

    let distance_attenuation = distance_attenuation(
        hit_to_light.length_squared(),
        1.0 / range.powf(2.0),
    );

    diffuse * diffuse_color * light.color() * distance_attenuation * n_o_l
}

fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
    let clamped_perceptual_roughness = perceptual_roughness.clamp(0.089, 1.0);

    clamped_perceptual_roughness * clamped_perceptual_roughness
}

fn diffuse_light(
    l: Vec3,
    v: Vec3,
    hit: Hit,
    roughness: f32,
    n_o_l: f32,
) -> f32 {
    let h = (l + v).normalize();
    let n_dot_v = hit.normal.dot(v).max(0.0001);
    let l_o_h = saturate(l.dot(h));

    fd_burley(roughness, n_dot_v, n_o_l, l_o_h)
}

fn fd_burley(roughness: f32, n_o_v: f32, n_o_l: f32, l_o_h: f32) -> f32 {
    fn f_schlick(f0: f32, f90: f32, v_o_h: f32) -> f32 {
        f0 + (f90 - f0) * (1.0 - v_o_h).powf(5.0)
    }

    let f90 = 0.5 + 2.0 * roughness * l_o_h * l_o_h;
    let light_scatter = f_schlick(1.0, f90, n_o_l);
    let view_scatter = f_schlick(1.0, f90, n_o_v);

    light_scatter * view_scatter * (1.0 / PI)
}

fn distance_attenuation(
    distance_square: f32,
    inverse_range_squared: f32,
) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = saturate(1.0 - factor * factor);
    let attenuation = smooth_factor * smooth_factor;

    attenuation * 1.0 / distance_square.max(0.0001)
}

fn saturate(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn reflect(e1: Vec3, e2: Vec3) -> Vec3 {
    e1 - 2.0 * e2.dot(e1) * e2
}

fn inverse_sqrt(x: f32) -> f32 {
    1.0 / x.sqrt()
}
