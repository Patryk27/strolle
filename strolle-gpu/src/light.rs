mod eval;

use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use self::eval::*;
use crate::{BvhStack, BvhView, Hit, Material, Noise, Ray, TrianglesView};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Light {
    /// x - position x
    /// y - position y
    /// z - position z
    /// w - radius
    pub d0: Vec4,

    /// x - color R
    /// y - color G
    /// z - color B
    /// w - range
    pub d1: Vec4,
}

impl Light {
    pub fn sun(pos: Vec3) -> Self {
        Self {
            d0: pos.extend(10.0),
            d1: Default::default(),
        }
    }

    pub fn center(&self) -> Vec3 {
        self.d0.xyz()
    }

    pub fn radius(&self) -> f32 {
        self.d0.w
    }

    pub fn color(&self) -> Vec3 {
        self.d1.xyz()
    }

    pub fn range(&self) -> f32 {
        self.d1.w
    }

    /// TODO check out https://blog.demofox.org/2020/05/16/using-blue-noise-for-raytraced-soft-shadows/
    /// TODO check out https://schuttejoe.github.io/post/arealightsampling/
    pub fn position(&self, noise: &mut Noise) -> Vec3 {
        self.center() + self.radius() * noise.sample_sphere()
    }

    pub fn contribution(
        &self,
        material: Material,
        hit: Hit,
        ray: Ray,
        albedo: Vec3,
    ) -> LightContribution {
        let roughness =
            perceptual_roughness_to_roughness(material.perceptual_roughness);

        let hit_to_light = self.center() - hit.point;
        let diffuse_color = albedo * (1.0 - material.metallic);
        let v = -ray.direction();
        let n_dot_v = hit.normal.dot(v).max(0.0001);
        let r = reflect(-v, hit.normal);

        let f0 = 0.16
            * material.reflectance
            * material.reflectance
            * (1.0 - material.metallic)
            + albedo * material.metallic;

        let range = self.range();

        let l = hit_to_light.normalize();
        let n_o_l = saturate(hit.normal.dot(l));

        let diffuse = diffuse_light(l, v, hit, roughness, n_o_l);
        let center_to_ray = hit_to_light.dot(r) * r - hit_to_light;

        let closest_point = hit_to_light
            + center_to_ray
                * saturate(
                    self.radius()
                        * inverse_sqrt(center_to_ray.dot(center_to_ray)),
                );

        let l_spec_length_inverse =
            inverse_sqrt(closest_point.dot(closest_point));

        let normalization_factor = roughness
            / saturate(
                roughness + (self.radius() * 0.5 * l_spec_length_inverse),
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

        let diffuse = diffuse
            * diffuse_color
            * self.color()
            * distance_attenuation
            * n_o_l;

        let specular = specular * self.color() * distance_attenuation * n_o_l;

        LightContribution { diffuse, specular }
    }

    pub fn visibility(
        &self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        stack: BvhStack,
        noise: &mut Noise,
        hit: Hit,
    ) -> f32 {
        let is_occluded = {
            let light_pos = self.position(noise);
            let light_to_hit = hit.point - light_pos;

            let shadow_ray = Ray::new(light_pos, light_to_hit.normalize());
            let max_distance = light_to_hit.length();

            shadow_ray.trace_any(local_idx, triangles, bvh, stack, max_distance)
        };

        if is_occluded {
            0.0
        } else {
            1.0
        }
    }
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct LightId(u32);

impl LightId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct LightContribution {
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl LightContribution {
    pub fn sum(&self) -> Vec3 {
        self.diffuse + self.specular
    }
}
