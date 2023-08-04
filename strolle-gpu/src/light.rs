use core::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec4, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::Sampler;

use crate::{
    BlueNoise, BvhStack, BvhView, DiffuseBrdf, F32Ext, Hit, MaterialsView,
    Normal, Ray, SpecularBrdf, Tex, TrianglesView, Vec3Ext, WhiteNoise,
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

    pub fn sun(pos: Vec3) -> Self {
        Self {
            d0: pos.extend(10.0),
            d1: Default::default(),
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

    /// Returns demodulated contribution of this light on given hit point.
    ///
    /// Note that this function doesn't perform visibility check (see:
    /// [`Self::visibility()`]).
    pub fn contribution(&self, hit: Hit) -> LightContribution {
        fn distance_attenuation(
            distance_square: f32,
            inverse_range_squared: f32,
        ) -> f32 {
            let factor = distance_square * inverse_range_squared;
            let smooth_factor = (1.0 - factor * factor).saturate();
            let attenuation = smooth_factor * smooth_factor;

            attenuation / distance_square.max(0.0001)
        }

        let cone_factor = if self.is_point() {
            1.0
        } else {
            let angle = self
                .spot_direction()
                .angle_between(hit.point - self.center());

            (1.0 - (angle / self.spot_angle()).powf(3.0)).saturate()
        };

        if cone_factor < 0.001 {
            return Default::default();
        }

        // ---

        let hit_to_light = self.center() - hit.point;

        let distance_factor = distance_attenuation(
            hit_to_light.length_squared(),
            1.0 / self.range().sqr(),
        );

        if distance_factor < 0.0001 {
            return Default::default();
        }

        // ---

        let v = (hit.origin - hit.point).normalize();
        let l = hit_to_light.normalize();
        let n = hit.gbuffer.normal;
        let n_o_l = n.dot(l).saturate();

        let diffuse = DiffuseBrdf::new(&hit.gbuffer).eval(l, v, n, n_o_l);

        let specular = {
            let n_o_v = n.dot(v).max(0.0001);
            let r = (-v).reflect(n);

            let center_to_ray = hit_to_light.dot(r) * r - hit_to_light;

            let closest_point = {
                let t = self.radius()
                    * center_to_ray.dot(center_to_ray).inverse_sqrt();

                hit_to_light + center_to_ray * t.saturate()
            };

            let l_spec_length_inverse =
                closest_point.dot(closest_point).inverse_sqrt();

            let i_roughness = {
                let t = hit.gbuffer.clamped_roughness()
                    + self.radius() * 0.5 * l_spec_length_inverse;

                hit.gbuffer.clamped_roughness() / t.saturate()
            };

            let intensity = i_roughness * i_roughness;

            let l = closest_point * l_spec_length_inverse;
            let h = (l + v).normalize();
            let n_o_l = n.dot(l).saturate();
            let n_o_h = n.dot(h).saturate();
            let l_o_h = l.dot(h).saturate();

            intensity
                * SpecularBrdf::new(&hit.gbuffer)
                    .eval(n_o_v, n_o_l, n_o_h, l_o_h)
        };

        let diffuse =
            diffuse * self.color() * distance_factor * cone_factor * n_o_l;

        let specular =
            specular * self.color() * distance_factor * cone_factor * n_o_l;

        LightContribution { diffuse, specular }
    }

    /// Casts a shadow ray and returns 0.0 if this light is occluded or 1.0 if
    /// this light is visible from given hit point.
    ///
    /// See also: [`Self::visibility_bnoise()`].
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
        let light_pos = self.center() + self.radius() * wnoise.sample_sphere();
        let light_to_hit = hit_point - light_pos;
        let ray = Ray::new(light_pos, light_to_hit.normalize());
        let distance = light_to_hit.length();

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

    /// Like [`Self::visiblity()`] but using blue noise; we use this for direct
    /// lightning because blue noise yields more useful samples.
    #[allow(clippy::too_many_arguments)]
    pub fn visibility_bnoise(
        &self,
        local_idx: u32,
        stack: BvhStack,
        triangles: TrianglesView,
        bvh: BvhView,
        materials: MaterialsView,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        bnoise: BlueNoise,
        hit: Hit,
    ) -> f32 {
        let to_light = self.center() - hit.point;
        let light_dir = to_light.normalize();
        let light_distance = to_light.length();
        let light_radius = self.radius() / light_distance;
        let (light_tangent, light_bitangent) = light_dir.any_orthonormal_pair();

        let disk_point = {
            let sample = bnoise.first_sample();
            let angle = 2.0 * PI * sample.x;
            let radius = sample.y.sqrt();

            vec2(angle.sin(), angle.cos()) * radius * light_radius
        };

        let ray_dir = light_dir
            + disk_point.x * light_tangent
            + disk_point.y * light_bitangent;

        let ray_dir = ray_dir.normalize();
        let ray = Ray::new(hit.point, ray_dir);

        let is_occluded = ray.intersect(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            light_distance,
        );

        if is_occluded {
            0.0
        } else {
            1.0
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
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

#[derive(Clone, Copy, Default)]
pub struct LightContribution {
    pub diffuse: Vec3,
    pub specular: Vec3,
}

impl LightContribution {
    pub fn sum(&self) -> Vec3 {
        self.diffuse + self.specular
    }
}
