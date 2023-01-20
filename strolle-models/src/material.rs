use core::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
#[cfg(not(target_arch = "spirv"))]
use glam::vec4;
use glam::{Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::float::Float;
use spirv_std::{Image, Sampler};

use crate::{
    BvhTraversingStack, BvhView, Hit, Light, LightId, LightsView, Noise, Ray,
    TrianglesView, World, MAX_IMAGES,
};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    base_color: Vec4,
    base_color_texture: u32,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    refraction: f32,
    reflectivity: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Material {
    #[allow(clippy::too_many_arguments)]
    pub fn shade(
        &self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        lights: LightsView,
        world: &World,
        images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
        samplers: &[Sampler; MAX_IMAGES],
        stack: BvhTraversingStack,
        ray: Ray,
        hit: Hit,
        noise: &mut Noise,
    ) -> (Vec3, Vec3) {
        let albedo = self.albedo(images, samplers, hit);

        let shade = {
            let mut shade = Vec3::ZERO;
            let mut light_id = 0;

            while light_id < world.light_count {
                let light = lights.get(LightId::new(light_id));

                shade += self.shade_light(
                    local_idx, triangles, bvh, stack, ray, hit, albedo, light,
                    noise,
                );

                light_id += 1;
            }

            shade
        };

        (albedo, shade)
    }

    fn albedo(
        &self,
        images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
        samplers: &[Sampler; MAX_IMAGES],
        hit: Hit,
    ) -> Vec3 {
        if self.base_color_texture == u32::MAX {
            self.base_color
        } else {
            let image = images[self.base_color_texture as usize];
            let sampler = samplers[self.base_color_texture as usize];

            self.base_color
                * image.sample_by_lod::<_, Vec4>(sampler, hit.uv, 0.0)
        }
        .xyz()
    }

    // TODO: Optimize a lot of these calculations can be done once per material
    #[allow(clippy::too_many_arguments)]
    fn shade_light(
        &self,
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
            perceptual_roughness_to_roughness(self.perceptual_roughness);

        let hit_to_light = light.center() - hit.point;
        let diffuse_color = albedo * (1.0 - self.metallic);
        let v = -ray.direction();
        let n_dot_v = hit.normal.dot(v).max(0.0001);
        let r = reflect(-v, hit.normal);

        let f0 =
            0.16 * self.reflectance * self.reflectance * (1.0 - self.metallic)
                + albedo * self.metallic;

        let range = light.range();

        let l = hit_to_light.normalize();
        let n_o_l = saturate(hit.normal.dot(l));

        let diffuse = diffuse_light(l, v, hit, roughness, n_o_l);
        let center_to_ray = hit_to_light.dot(r) * r - hit_to_light;

        let closest_point = hit_to_light
            + center_to_ray
                * saturate(
                    light.radius()
                        * inverse_sqrt(center_to_ray.dot(center_to_ray)),
                );

        let l_spec_length_inverse =
            inverse_sqrt(closest_point.dot(closest_point));

        let normalization_factor = roughness
            / saturate(
                roughness + (light.radius() * 0.5 * l_spec_length_inverse),
            );

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
}

fn perceptual_roughness_to_roughness(perceptual_roughness: f32) -> f32 {
    // clamp perceptual roughness to prevent precision problems
    // According to Filament design 0.089 is recommended for mobile
    // Filament uses 0.045 for non-mobile
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

// Thanks to https://google.github.io/filament/Filament.html
fn fd_burley(roughness: f32, n_o_v: f32, n_o_l: f32, l_o_h: f32) -> f32 {
    fn f_schlick(f0: f32, f90: f32, v_o_h: f32) -> f32 {
        f0 + (f90 - f0) * (1.0 - v_o_h).powf(5.0)
    }

    let f90 = 0.5 + 2.0 * roughness * l_o_h * l_o_h;
    let light_scatter = f_schlick(1.0, f90, n_o_l);
    let view_scatter = f_schlick(1.0, f90, n_o_v);

    light_scatter * view_scatter * (1.0 / PI)
}

// Normal distribution function (specular D)
// Based on https://google.github.io/filament/Filament.html#citation-walter07

// D_GGX(h,α) = α^2 / { π ((n⋅h)^2 (α2−1) + 1)^2 }

// Simple implementation, has precision problems when using fp16 instead of fp32
// see https://google.github.io/filament/Filament.html#listing_speculardfp16
fn d_ggx(roughness: f32, n_o_h: f32) -> f32 {
    let one_minus_no_h_squared = 1.0 - n_o_h * n_o_h;
    let a = n_o_h * roughness;
    let k = roughness / (one_minus_no_h_squared + a * a);

    k * k * (1.0 / PI)
}

// Visibility function (Specular G)
// V(v,l,a) = G(v,l,α) / { 4 (n⋅v) (n⋅l) }
// such that f_r becomes
// f_r(v,l) = D(h,α) V(v,l,α) F(v,h,f0)
// where
// V(v,l,α) = 0.5 / { n⋅l sqrt((n⋅v)^2 (1−α2) + α2) + n⋅v sqrt((n⋅l)^2 (1−α2) + α2) }
// Note the two sqrt's, that may be slow on mobile, see https://google.github.io/filament/Filament.html#listing_approximatedspecularv
fn v_smith_ggx_correlated(roughness: f32, n_o_v: f32, n_o_l: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambda_v = n_o_l * f32::sqrt((n_o_v - a2 * n_o_v) * n_o_v + a2);
    let lambda_l = n_o_v * f32::sqrt((n_o_l - a2 * n_o_l) * n_o_l + a2);

    0.5 / (lambda_v + lambda_l)
}

fn fresnel(f0: Vec3, l_o_h: f32) -> Vec3 {
    // f_90 suitable for ambient occlusion
    // see https://google.github.io/filament/Filament.html#lighting/occlusion
    let f90 = saturate(f0.dot(Vec3::splat(50.0 * 0.33)));

    f_schlick_vec(f0, f90, l_o_h)
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_o_h: f32) -> Vec3 {
    // not using mix to keep the vec3 and float versions identical
    f0 + (f90 - f0) * f32::powf(1.0 - v_o_h, 5.0)
}

// Thanks to Bevy's `pbr_lightning.wgsl`
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

#[cfg(not(target_arch = "spirv"))]
impl Material {
    pub fn with_base_color(mut self, base_color: Vec4) -> Self {
        self.base_color = base_color;
        self
    }

    pub fn with_base_color_texture(
        mut self,
        base_color_texture: impl Into<Option<u32>>,
    ) -> Self {
        self.base_color_texture = base_color_texture.into().unwrap_or(u32::MAX);
        self
    }

    pub fn with_perceptual_roughness(
        mut self,
        perceptual_roughness: f32,
    ) -> Self {
        self.perceptual_roughness = perceptual_roughness;
        self
    }

    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    pub fn with_reflectance(mut self, reflectance: f32) -> Self {
        self.reflectance = reflectance;
        self
    }

    pub fn with_refraction(mut self, refraction: f32) -> Self {
        self.refraction = refraction;
        self
    }

    pub fn with_reflectivity(mut self, reflectivity: f32) -> Self {
        self.reflectivity = reflectivity;
        self
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: vec4(1.0, 1.0, 1.0, 1.0),
            base_color_texture: u32::MAX,
            perceptual_roughness: 0.0,
            metallic: 0.0,
            reflectance: 0.0,
            refraction: 1.0,
            reflectivity: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct MaterialId(u32);

impl MaterialId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn get_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}
