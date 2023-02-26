#![no_std]

use core::f32::consts::PI;

use spirv_std::glam::{
    vec2, vec3, UVec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles,
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
        let mut noise = Noise::new(params.seed, global_idx);

        let material = materials.get(MaterialId::new(hit.material_id));
        let albedo = material.albedo(images, samplers, hit);

        let mut shade = Vec3::ZERO;
        let mut light_id = 0;

        while light_id < world.light_count {
            let light = lights.get(LightId::new(light_id));

            shade += shade_light(
                material,
                local_idx,
                triangles,
                bvh,
                stack,
                ray,
                hit,
                albedo.xyz(),
                light,
                &mut noise,
            );

            light_id += 1;
        }

        color = shade * ray_throughput;
        normal = hit.normal.extend(f32::from_bits(hit.material_id));

        (next_ray_origin, next_ray_direction, next_ray_throughput) = next_ray(
            material,
            &mut noise,
            ray.direction(),
            ray_throughput,
            hit,
            albedo,
        );
    } else {
        color = camera.clear_color() * ray_throughput;
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
        } else {
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

#[allow(clippy::too_many_arguments)]
fn shade_light(
    material: Material,
    local_idx: u32,
    triangles: TrianglesView,
    bvh: BvhView,
    stack: BvhTraversingStack,
    ray: Ray,
    hit: Hit,
    albedo: Vec3,
    light: Light,
    noise: &mut Noise,
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

fn next_ray(
    material: Material,
    noise: &mut Noise,
    ray_direction: Vec3,
    ray_throughput: Vec3,
    hit: Hit,
    albedo: Vec4,
) -> (Vec3, Vec3, Vec3) {
    let specular_chance = material.reflectivity;
    let refraction_chance = 1.0 - albedo.w;

    let do_specular;
    let do_refraction;
    let ray_probability;

    let chance = noise.sample();

    if specular_chance > 0.0 && chance < specular_chance {
        do_specular = 1.0;
        do_refraction = 0.0;
        ray_probability = specular_chance;
    } else if refraction_chance > 0.0
        && chance < specular_chance + refraction_chance
    {
        do_specular = 0.0;
        do_refraction = 1.0;
        ray_probability = refraction_chance;
    } else {
        do_specular = 0.0;
        do_refraction = 0.0;
        ray_probability = 1.0 - specular_chance - refraction_chance;
    }

    let next_ray_origin = if do_refraction > 0.0 {
        hit.point + ray_direction * 0.1
    } else {
        hit.point
    };

    let next_ray_direction = {
        let diffuse_direction =
            (hit.normal + noise.sample_hemisphere()).normalize();

        let specular_direction = reflect(ray_direction, hit.normal);

        let refraction_direction = {
            let mut cos_incident_angle = hit.normal.dot(-ray_direction);

            let eta = if cos_incident_angle > 0.0 {
                material.refraction
            } else {
                1.0 / material.refraction
            };

            let refraction_coeff =
                1.0 - (1.0 - cos_incident_angle.powi(2)) / eta.powi(2);

            if refraction_coeff < 0.0 {
                // TODO
            }

            let mut normal = hit.normal;
            let cos_transmitted_angle = refraction_coeff.sqrt();

            if cos_incident_angle < 0.0 {
                normal = -normal;
                cos_incident_angle = -cos_incident_angle;
            }

            ray_direction / eta
                - normal * (cos_transmitted_angle - cos_incident_angle / eta)
        };

        let refraction_direction = refraction_direction
            .lerp(
                (-hit.normal + noise.sample_hemisphere()).normalize(),
                material.perceptual_roughness.powi(2),
            )
            .normalize();

        diffuse_direction
            .lerp(specular_direction, do_specular)
            .lerp(refraction_direction, do_refraction)
    };

    let next_ray_throughput = if do_refraction > 0.0 {
        ray_throughput
    } else {
        ray_throughput * albedo.xyz()
    };

    let next_ray_throughput = next_ray_throughput / ray_probability;

    (next_ray_origin, next_ray_direction, next_ray_throughput)
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
