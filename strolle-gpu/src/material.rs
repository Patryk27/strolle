use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::Sampler;

use crate::Tex;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    pub base_color: Vec4,
    pub base_color_texture: Vec4,
    pub emissive: Vec4,
    pub emissive_texture: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub ior: f32,
    pub metallic_roughness_texture: Vec4,
    pub normal_map_texture: Vec4,
}

impl Material {
    /// Adjusts material so that it's ready for computing indirect lighting.
    pub fn regularize(&mut self) {
        self.roughness = self.roughness.max(0.75 * 0.75);
    }

    pub fn base_color(
        &self,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        hit_uv: Vec2,
    ) -> Vec4 {
        Self::sample_atlas(
            atlas_tex,
            atlas_sampler,
            hit_uv,
            self.base_color,
            self.base_color_texture,
        )
    }
    pub fn metallic_roughness(
        &self,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        hit_uv: Vec2,
    ) -> Vec2 {
        if self.metallic_roughness_texture == Vec4::ZERO {
            Vec4::new(1.0, self.roughness, self.metallic, 1.0).zy()
        } else {
            Self::sample_atlas(
                atlas_tex,
                atlas_sampler,
                hit_uv,
                Vec4::ONE,
                self.metallic_roughness_texture,
            )
            .zy()
        }
    }
    pub fn emissive(
        &self,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        hit_uv: Vec2,
    ) -> Vec3 {
        Self::sample_atlas(
            atlas_tex,
            atlas_sampler,
            hit_uv,
            self.emissive,
            self.emissive_texture,
        )
        .xyz()
    }

    fn sample_atlas(
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        mut hit_uv: Vec2,
        multiplier: Vec4,
        texture: Vec4,
    ) -> Vec4 {
        // TODO this assumes the texture's sampler is configured to U/V-repeat
        //      which might not be the case; we should propagate sampler info up
        //      to here and decide
        let wrap = |t: f32| {
            if t > 0.0 {
                t % 1.0
            } else {
                1.0 - (-t % 1.0)
            }
        };

        if texture == Vec4::ZERO {
            multiplier
        } else {
            hit_uv.x = wrap(hit_uv.x);
            hit_uv.y = wrap(hit_uv.y);

            let uv = texture.xy() + hit_uv * texture.zw();

            multiplier * atlas_tex.sample_by_lod(*atlas_sampler, uv, 0.0)
        }
    }

    // TODO bring back
    //
    // pub fn normal(
    //     &self,
    //     hit_uv: Vec2,
    //     hit_normal: Vec3,
    //     hit_tangent: Vec4,
    // ) -> Vec3 {
    //     if self.normal_map_texture == u32::MAX {
    //         hit_normal
    //     } else {
    //         let normal_map_tex = unsafe {
    //             images.index_unchecked(self.normal_map_texture as usize)
    //         };

    //         let normal_map_sampler = unsafe {
    //             samplers.index_unchecked(self.normal_map_texture as usize)
    //         };

    //         let tangent = hit_tangent.xyz();
    //         let bitangent = hit_tangent.w * hit_normal.cross(tangent);

    //         let mapped_normal =
    //             normal_map_tex.sample(*normal_map_sampler, hit_uv);

    //         let mapped_normal = 2.0 * mapped_normal - 1.0;

    //         (mapped_normal.x * tangent
    //             + mapped_normal.y * bitangent
    //             + mapped_normal.z * hit_normal)
    //             .normalize()
    //     }
    // }
}

#[derive(Clone, Copy)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct MaterialId(u32);

impl MaterialId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}
