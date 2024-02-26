use core::f32::consts::PI;
use core::ops::Mul;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{
    DiffuseBrdf, F32Ext, Hit, Normal, Ray, SpecularBrdf, Vec3Ext, WhiteNoise,
};

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
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

    /// x - (as u32) light type
    /// y - if it's a spot light: direction
    /// z - if it's a spot light: direction
    /// w - if it's a spot light: angle
    pub d2: Vec4,

    /// x - (as u32) see the "slot" functions below
    pub d3: Vec4,

    // Light's data from the previous frame
    pub prev_d0: Vec4,
    pub prev_d1: Vec4,
    pub prev_d2: Vec4,
}

impl Light {
    pub const TYPE_NONE: u32 = 0;
    pub const TYPE_POINT: u32 = 1;
    pub const TYPE_SPOT: u32 = 2;

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
            d3: Default::default(),
            prev_d0: Default::default(),
            prev_d1: Default::default(),
            prev_d2: Default::default(),
        }
    }

    pub fn center(self) -> Vec3 {
        self.d0.xyz()
    }

    pub fn radius(self) -> f32 {
        self.d0.w
    }

    pub fn color(self) -> Vec3 {
        self.d1.xyz()
    }

    pub fn range(self) -> f32 {
        self.d1.w
    }

    pub fn contains(self, point: Vec3) -> bool {
        self.center().distance(point) <= self.radius()
    }

    fn ty(self) -> u32 {
        self.d2.x.to_bits()
    }

    pub fn is_none(self) -> bool {
        self.ty() == Self::TYPE_NONE
    }

    pub fn is_point(self) -> bool {
        self.ty() == Self::TYPE_POINT
    }

    pub fn spot_dir(self) -> Vec3 {
        Normal::decode(self.d2.yz())
    }

    pub fn spot_angle(self) -> f32 {
        self.d2.w
    }

    pub fn is_slot_remapped(self) -> bool {
        self.d3.x.to_bits() > 0 && self.d3.x.to_bits() != 0xcafebabe
    }

    pub fn slot_remapped_to(self) -> LightId {
        LightId::new(self.d3.x.to_bits() - 1)
    }

    pub fn remap_slot(&mut self, id: LightId) {
        self.d3.x = f32::from_bits(id.get() + 1);
    }

    pub fn is_slot_killed(self) -> bool {
        self.d3.x.to_bits() == 0xcafebabe
    }

    pub fn kill_slot(&mut self) {
        self.d3.x = f32::from_bits(0xcafebabe);
    }

    pub fn clear_slot(&mut self) {
        self.d3.x = f32::from_bits(0);
    }

    pub fn commit(&mut self) {
        self.prev_d0 = self.d0;
        self.prev_d1 = self.d1;
        self.prev_d2 = self.d2;
    }

    pub fn rollback(&mut self) {
        self.d0 = self.prev_d0;
        self.d1 = self.prev_d1;
        self.d2 = self.prev_d2;
    }

    pub fn radiance(self, hit: Hit) -> LightRadiance {
        let l = self.center() - hit.point;

        let f_angle = if self.is_point() {
            1.0
        } else {
            let angle =
                self.spot_dir().angle_between(hit.point - self.center());

            (1.0 - (angle / self.spot_angle()).powf(3.0)).saturate()
        };

        let f_dist = if self.range() == f32::INFINITY {
            1.0
        } else {
            let l2 = l.length_squared();
            let inv_r2 = 1.0 / self.range().sqr();

            let factor = l2 * inv_r2;
            let smooth_factor = (1.0 - factor * factor).saturate();
            let attenuation = smooth_factor * smooth_factor;

            attenuation / l2.max(0.0001)
        };

        let f_cosine = hit.gbuffer.normal.dot(l.normalize()).saturate();

        let diff_brdf = DiffuseBrdf::new(hit.gbuffer).eval();

        let spec_brdf = {
            let v = -hit.dir;
            let n = hit.gbuffer.normal;
            let r = (-v).reflect(n);

            let center_to_ray = l.dot(r) * r - l;

            let closest_point = {
                let t = self.radius()
                    * center_to_ray.dot(center_to_ray).inverse_sqrt();

                l + center_to_ray * t.saturate()
            };

            let l_spec_length_inverse =
                closest_point.dot(closest_point).inverse_sqrt();

            let i_roughness = {
                let t = hit.gbuffer.clamped_roughness()
                    + self.radius() * 0.5 * l_spec_length_inverse;

                hit.gbuffer.clamped_roughness() / t.saturate()
            };

            let intensity = i_roughness.sqr();
            let l = closest_point * l_spec_length_inverse;

            intensity * SpecularBrdf::new(hit.gbuffer).eval(l, v)
        };

        LightRadiance {
            radiance: self.color() * f_angle * f_dist * f_cosine,
            diff_brdf,
            spec_brdf,
        }
    }

    pub fn ray_wnoise(self, noise: &mut WhiteNoise, hit_point: Vec3) -> Ray {
        let light_pos = self.center() + self.radius() * noise.sample_sphere();
        let light_to_hit = hit_point - light_pos;

        Ray::new(light_pos, light_to_hit.normalize())
            .with_len(light_to_hit.length())
    }

    pub fn ray_bnoise(self, sample: Vec2, hit_point: Vec3) -> Ray {
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
            .with_len(light_distance)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct LightId(u32);

impl LightId {
    pub const fn new(id: u32) -> Self {
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

#[derive(Clone, Copy, Default)]
pub struct LightRadiance {
    pub radiance: Vec3,
    pub diff_brdf: Vec3,
    pub spec_brdf: Vec3,
}

impl LightRadiance {
    pub fn sum(self) -> Vec3 {
        self.radiance * (self.diff_brdf + self.spec_brdf)
    }
}

impl Mul<f32> for LightRadiance {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.radiance *= rhs;
        self
    }
}
