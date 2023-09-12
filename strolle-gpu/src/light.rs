use bytemuck::{Pod, Zeroable};
use glam::{vec4, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::Sampler;

use crate::{
    BvhStack, BvhView, DiffuseBrdf, F32Ext, Hit, MaterialsView, Normal, Ray,
    Tex, TrianglesView, WhiteNoise,
};

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
            d0: position.extend(100.0),
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

    /// Returns demodulated¹ radiance of this light on given hit.
    ///
    /// ¹ without taking into account the hit-material
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

    /// Returns contribution (i.e. "the surface color") of this light on given
    /// hit.
    pub fn contribution(&self, hit: Hit) -> Vec3 {
        let l = (self.center() - hit.point).normalize();
        let v = (hit.origin - hit.point).normalize();
        let diffuse = DiffuseBrdf::new(&hit.gbuffer).evaluate(l, v).radiance;

        self.radiance(hit) * diffuse
    }

    /// Casts a shadow ray and returns 0.0 if this light is occluded or 1.0 if
    /// this light is visible from given hit point.
    #[allow(clippy::too_many_arguments)]
    pub fn visibility(
        &self,
        local_idx: u32,
        stack: BvhStack,
        triangles: TrianglesView,
        bvh: BvhView,
        materials: MaterialsView,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        wnoise: &mut WhiteNoise,
        hit_point: Vec3,
    ) -> f32 {
        let (ray, distance) = self.ray(wnoise, hit_point);

        let is_occluded = ray.intersect(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            distance,
        );

        if is_occluded {
            0.0
        } else {
            1.0
        }
    }

    pub fn ray(&self, wnoise: &mut WhiteNoise, hit_point: Vec3) -> (Ray, f32) {
        let light_pos = self.center() + self.radius() * wnoise.sample_sphere();
        let light_to_hit = hit_point - light_pos;
        let ray = Ray::new(light_pos, light_to_hit.normalize());
        let distance = light_to_hit.length();

        (ray, distance)
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
}
