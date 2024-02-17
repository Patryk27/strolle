use core::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{DiffuseBrdf, F32Ext, Hit, Normal, Ray, WhiteNoise};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Light {
    /// x - position x
    /// y - position y
    /// z - position z
    /// w - radius
    pub d0: Vec4,

    /// x - color r
    /// y - color g
    /// z - color b
    /// w - range
    pub d1: Vec4,

    /// x - (as u32) light type: 0 - point light, 1 - spot light
    /// y - if it's a spot light: direction
    /// z - if it's a spot light: direction
    /// w - if it's a spot light: angle
    pub d2: Vec4,
}

impl Light {
    pub const TYPE_POINT: u32 = 0;
    pub const TYPE_SPOT: u32 = 1;

    pub fn sun(position: Vec3, color: Vec3) -> Self {
        Self {
            // TODO incorrect
            d0: position.extend(25.0),
            d1: color.extend(f32::INFINITY),
            d2: vec4(
                f32::from_bits(Self::TYPE_POINT),
                Default::default(),
                Default::default(),
                Default::default(),
            ),
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

    pub fn is_point(&self) -> bool {
        self.d2.x.to_bits() == Self::TYPE_POINT
    }

    pub fn spot_direction(&self) -> Vec3 {
        Normal::decode(self.d2.yz())
    }

    pub fn spot_angle(&self) -> f32 {
        self.d2.w
    }

    pub fn radiance(&self, hit: Hit) -> Vec3 {
        let l = self.center() - hit.point;

        let conical_factor = if self.is_point() {
            1.0
        } else {
            let angle = self
                .spot_direction()
                .angle_between(hit.point - self.center());

            (1.0 - (angle / self.spot_angle()).powf(3.0)).saturate()
        };

        let distance_factor = {
            fn distance_attenuation(
                distance_square: f32,
                inverse_range_squared: f32,
            ) -> f32 {
                let factor = distance_square * inverse_range_squared;
                let smooth_factor = (1.0 - factor * factor).saturate();
                let attenuation = smooth_factor * smooth_factor;

                attenuation / distance_square.max(0.0001)
            }

            if self.range() == f32::INFINITY {
                1.0
            } else {
                distance_attenuation(
                    l.length_squared(),
                    1.0 / self.range().sqr(),
                )
            }
        };

        let cosine_factor = hit.gbuffer.normal.dot(l.normalize()).saturate();

        self.color() * distance_factor * conical_factor * cosine_factor
    }

    pub fn contribution(&self, hit: Hit) -> Vec3 {
        let brdf = DiffuseBrdf::new(&hit.gbuffer).evaluate();

        self.radiance(hit) * brdf.radiance / brdf.probability
    }

    pub fn ray_wnoise(&self, noise: &mut WhiteNoise, hit_point: Vec3) -> Ray {
        let light_pos = self.center() + self.radius() * noise.sample_sphere();
        let light_to_hit = hit_point - light_pos;

        Ray::new(light_pos, light_to_hit.normalize())
            .with_length(light_to_hit.length())
    }

    pub fn ray_bnoise(&self, sample: Vec2, hit_point: Vec3) -> Ray {
        let to_light = self.center() - hit_point;
        let light_dir = to_light.normalize();
        let light_distance = to_light.length();
        let light_radius = self.radius() / light_distance;
        let (light_tangent, light_bitangent) = light_dir.any_orthonormal_pair();

        let disk_point = {
            let angle = 2.0 * PI * sample.x;
            let radius = sample.y.sqrt();

            vec2(angle.sin(), angle.cos()) * radius * light_radius
        };

        let ray_dir = light_dir
            + disk_point.x * light_tangent
            + disk_point.y * light_bitangent;

        let ray_dir = ray_dir.normalize();

        Ray::new(hit_point + ray_dir * light_distance, -ray_dir)
            .with_length(light_distance)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct LightId(u32);

impl LightId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn sky() -> Self {
        Self::new(u32::MAX)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    #[cfg(not(target_arch = "spirv"))]
    pub fn get_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}
