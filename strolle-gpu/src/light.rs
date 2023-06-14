mod eval;

use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

pub use self::eval::*;
use crate::{
    BvhTraversingStack, BvhView, Hit, Material, Noise, Ray, TrianglesView,
};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Light {
    /// x - position x
    /// y - position y
    /// z - position z
    /// w - unused
    pub d0: Vec4,

    /// x - color R
    /// y - color G
    /// z - color B
    /// w - range
    pub d1: Vec4,
}

impl Light {
    pub fn center(&self) -> Vec3 {
        self.d0.truncate()
    }

    pub fn position(&self, _noise: &mut Noise) -> Vec3 {
        self.center()
        // self.center() + self.radius() * noise.sample_sphere() TODO
    }

    pub fn color(&self) -> Vec3 {
        self.d1.truncate()
    }

    pub fn range(&self) -> f32 {
        self.d1.w
    }

    // TODO: Make configurable
    pub fn radius(&self) -> f32 {
        0.1
    }

    #[allow(clippy::too_many_arguments)]
    pub fn eval(
        &self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        stack: BvhTraversingStack,
        noise: &mut Noise,
        material: Material,
        hit: Hit,
        ray: Ray,
        albedo: Vec3,
    ) -> Vec3 {
        let is_occluded = {
            let light_pos = self.position(noise);
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

        let diffuse = diffuse * diffuse_color;

        (diffuse + specular) * self.color() * distance_attenuation * n_o_l
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
