#![no_std]

use core::f32::consts::PI;

use spirv_std::glam::{UVec2, UVec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
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
    params: &RayShadingPassParams,
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
    #[spirv(descriptor_set = 1, binding = 5)]
    pending_directs: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 6)]
    pending_indirects: &Image!(2D, format = rgba16f, sampled = false),
    #[spirv(descriptor_set = 1, binding = 7)]
    pending_normals: &Image!(2D, format = rgba16f, sampled = false),
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
        VoxelsView::new(voxels),
        pending_directs,
        pending_indirects,
        pending_normals,
    )
}

#[allow(clippy::too_many_arguments)]
fn main_inner(
    global_id: UVec2,
    local_idx: u32,
    params: &RayShadingPassParams,
    stack: BvhTraversingStack,
    triangles: TrianglesView,
    bvh: BvhView,
    lights: LightsView,
    materials: MaterialsView,
    images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
    samplers: &[Sampler; MAX_IMAGES],
    world: &World,
    camera: &Camera,
    primary_hits_d0: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d1: &Image!(2D, format = rgba32f, sampled = false),
    primary_hits_d2: &Image!(2D, format = rgba32f, sampled = false),
    voxels: VoxelsView,
    pending_directs: &Image!(2D, format = rgba16f, sampled = false),
    pending_indirects: &Image!(2D, format = rgba16f, sampled = false),
    pending_normals: &Image!(2D, format = rgba16f, sampled = false),
) {
    let mut noise = Noise::new(params.seed, global_id.x, global_id.y);
    let ray = camera.ray(global_id);

    let hit = Hit::from_primary(
        primary_hits_d0.read(global_id),
        primary_hits_d1.read(global_id),
        primary_hits_d2.read(global_id),
        ray,
    );

    let direct;
    let indirect;
    let normal;

    if hit.is_some() {
        let material = materials.get(MaterialId::new(hit.material_id));
        let albedo = material.albedo(images, samplers, hit).xyz();

        direct = compute_direct_lightning(
            local_idx, triangles, bvh, lights, world, stack, &mut noise, hit,
            ray, material, albedo,
        );

        indirect = compute_indirect_lightning(
            params, world, voxels, &mut noise, hit, albedo,
        );

        normal = hit.flat_normal.extend(f32::from_bits(hit.material_id));
    } else {
        direct = camera.clear_color();
        indirect = Vec3::ZERO;
        normal = Vec4::ZERO;
    }

    unsafe {
        pending_directs.write(global_id, direct.extend(1.0));
        pending_indirects.write(global_id, indirect.extend(1.0));
        pending_normals.write(global_id, normal);
    }
}

fn compute_indirect_lightning(
    params: &RayShadingPassParams,
    world: &World,
    voxels: VoxelsView,
    noise: &mut Noise,
    hit: Hit,
    albedo: Vec3,
) -> Vec3 {
    let mut indirect = Vec4::ZERO;
    let mut sample_idx = 0;

    let (u, v) = hit.flat_normal.any_orthonormal_pair();

    while sample_idx < 3 {
        let angle = 2.0 * PI * noise.sample();
        let radius = 2.0 * VOXEL_SIZE * noise.sample().sqrt();
        let delta = v * radius * angle.cos() + u * radius * angle.sin();

        let voxel =
            voxels.get(world.voxelize(hit.point + delta, hit.flat_normal));

        if voxel.is_fresh(params.frame) && voxel.is_nearby(hit.point) {
            indirect += voxel.scolor();
        }

        sample_idx += 1;
    }

    if indirect.w == 0.0 {
        let voxel = voxels.get(world.voxelize(hit.point, hit.flat_normal));

        if voxel.is_fresh(params.frame) && voxel.is_nearby(hit.point) {
            indirect += voxel.scolor();
        }
    }

    albedo * (indirect.xyz() / indirect.w.max(1.0))
}

fn compute_direct_lightning(
    local_idx: u32,
    triangles: TrianglesView,
    bvh: BvhView,
    lights: LightsView,
    world: &World,
    stack: BvhTraversingStack,
    noise: &mut Noise,
    hit: Hit,
    ray: Ray,
    material: Material,
    albedo: Vec3,
) -> Vec3 {
    let mut direct = Vec3::ZERO;
    let mut light_id = 0;

    while light_id < world.light_count {
        let light = lights.get(LightId::new(light_id));

        direct += compute_direct_lightning_for(
            local_idx, triangles, bvh, stack, noise, material, hit, ray,
            albedo, light,
        );

        light_id += 1;
    }

    direct
}

#[allow(clippy::too_many_arguments)]
fn compute_direct_lightning_for(
    local_idx: u32,
    triangles: TrianglesView,
    bvh: BvhView,
    stack: BvhTraversingStack,
    noise: &mut Noise,
    material: Material,
    hit: Hit,
    ray: Ray,
    albedo: Vec3,
    light: Light,
) -> Vec3 {
    let is_occluded = {
        let light_pos = light.position(noise);
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
    let n_dot_v = hit.normal.dot(v).max(0.0001);
    let r = reflect(-v, hit.normal);

    let f0 = 0.16
        * material.reflectance
        * material.reflectance
        * (1.0 - material.metallic)
        + albedo * material.metallic;

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

    let normalization_factor = roughness
        / saturate(roughness + (light.radius() * 0.5 * l_spec_length_inverse));

    let specular_intensity = normalization_factor * normalization_factor;

    let l = closest_point * l_spec_length_inverse;
    let h = (l + v).normalize();
    let n_o_l = saturate(hit.normal.dot(l));
    let n_o_h = saturate(hit.normal.dot(h));
    let l_o_h = saturate(l.dot(h));

    let specular = specular(
        f0,
        roughness,
        n_dot_v,
        n_o_l,
        n_o_h,
        l_o_h,
        specular_intensity,
    );

    let distance_attenuation = distance_attenuation(
        hit_to_light.length_squared(),
        1.0 / range.powf(2.0),
    );

    let diffuse = diffuse * diffuse_color;

    (diffuse + specular) * light.color() * distance_attenuation * n_o_l
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

fn specular(
    f0: Vec3,
    roughness: f32,
    n_o_v: f32,
    n_o_l: f32,
    n_o_h: f32,
    l_o_h: f32,
    specular_intensity: f32,
) -> Vec3 {
    let d = d_ggx(roughness, n_o_h);
    let v = v_smith_ggx_correlated(roughness, n_o_v, n_o_l);
    let f = fresnel(f0, l_o_h);

    (specular_intensity * d * v) * f
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

fn d_ggx(roughness: f32, n_o_h: f32) -> f32 {
    let one_minus_no_h_squared = 1.0 - n_o_h * n_o_h;
    let a = n_o_h * roughness;
    let k = roughness / (one_minus_no_h_squared + a * a);

    k * k * (1.0 / PI)
}

fn v_smith_ggx_correlated(roughness: f32, n_o_v: f32, n_o_l: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambda_v = n_o_l * f32::sqrt((n_o_v - a2 * n_o_v) * n_o_v + a2);
    let lambda_l = n_o_v * f32::sqrt((n_o_l - a2 * n_o_l) * n_o_l + a2);

    0.5 / (lambda_v + lambda_l)
}

fn fresnel(f0: Vec3, l_o_h: f32) -> Vec3 {
    let f90 = saturate(f0.dot(Vec3::splat(50.0 * 0.33)));

    f_schlick_vec(f0, f90, l_o_h)
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_o_h: f32) -> Vec3 {
    f0 + (f90 - f0) * f32::powf(1.0 - v_o_h, 5.0)
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
