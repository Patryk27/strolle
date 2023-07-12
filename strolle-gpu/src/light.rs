mod eval;

use core::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec3, vec4, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::Sampler;

use self::eval::*;
use crate::{
    BlueNoise, BvhStack, BvhView, F32Ext, Hit, Material, MaterialsView, Normal,
    Ray, Tex, TrianglesView, Vec3Ext, WhiteNoise,
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

    /// Evaluates this light on given material and return its contribution (i.e.
    /// unshaded color).
    ///
    /// Note that this function doesn't perform visibility check (see:
    /// [`Self::visibility()`]).
    pub fn contribution(
        &self,
        material: Material,
        hit: Hit,
        ray: Ray,
    ) -> LightContribution {
        let cone_factor = if self.is_point() {
            1.0
        } else {
            let angle = self
                .spot_direction()
                .angle_between(hit.point - self.center());

            (1.0 - (angle / self.spot_angle()).powf(3.0)).saturate()
        };

        if cone_factor < 0.01 {
            return Default::default();
        }

        let range = self.range();
        let hit_to_light = self.center() - hit.point;
        let v = (ray.origin() - hit.point).normalize();
        let l = hit_to_light.normalize();
        let n_o_l = hit.normal.dot(l).saturate();

        let diffuse = {
            let diffuse_color = Vec3::ONE * (1.0 - material.metallic);

            diffuse(l, v, hit, material.roughness, n_o_l) * diffuse_color
        };

        let specular = {
            let n_dot_v = hit.normal.dot(v).max(0.0001);
            let r = (-v).reflect(hit.normal);

            let f0 = 0.16
                * material.reflectance
                * material.reflectance
                * (1.0 - material.metallic)
                + Vec3::ONE * material.metallic;

            let center_to_ray = hit_to_light.dot(r) * r - hit_to_light;

            let closest_point = {
                let t = self.radius()
                    * center_to_ray.dot(center_to_ray).inverse_sqrt();

                hit_to_light + center_to_ray * t.saturate()
            };

            let l_spec_length_inverse =
                closest_point.dot(closest_point).inverse_sqrt();

            let normalization_factor = {
                let t = material.roughness
                    + self.radius() * 0.5 * l_spec_length_inverse;

                material.roughness / t.saturate()
            };

            let specular_intensity =
                normalization_factor * normalization_factor;

            let l = closest_point * l_spec_length_inverse;
            let h = (l + v).normalize();
            let n_o_l = hit.normal.dot(l).saturate();
            let n_o_h = hit.normal.dot(h).saturate();
            let l_o_h = l.dot(h).saturate();

            specular(
                f0,
                material.roughness,
                n_dot_v,
                n_o_l,
                n_o_h,
                l_o_h,
                specular_intensity,
            )
        };

        let distance_attenuation = distance_attenuation(
            hit_to_light.length_squared(),
            1.0 / range.powf(2.0),
        );

        let diffuse =
            diffuse * self.color() * distance_attenuation * n_o_l * cone_factor;

        let specular = specular
            * self.color()
            * distance_attenuation
            * n_o_l
            * cone_factor;

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
        hit: Hit,
    ) -> f32 {
        let light_pos = self.center() + self.radius() * wnoise.sample_sphere();
        let light_to_hit = hit.point - light_pos;
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
        let light_tangent = light_dir.cross(vec3(0.0, 1.0, 0.0)).normalize();
        let light_bitangent = light_tangent.cross(light_dir).normalize();

        let disk_point = {
            let sample = bnoise.first_sample();
            let angle = 2.0 * PI * sample.x;
            let length = sample.y.sqrt();

            vec2(angle.sin(), angle.cos()) * length * light_radius
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
    pub fn with_albedo(mut self, albedo: Vec3) -> Self {
        // TODO that's not correct (we're missing information about material's
        //      metallicness)
        self.diffuse *= albedo;

        // TODO support specular
        self
    }

    pub fn sum(&self) -> Vec3 {
        self.diffuse + self.specular
    }
}
