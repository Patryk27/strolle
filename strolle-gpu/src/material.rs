use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4, Vec4Swizzles};
use spirv_std::{Image, Sampler};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    pub base_color: Vec4,
    pub base_color_texture: Vec4,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub refraction: f32,
    pub reflectivity: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub normal_map_texture: Vec4,
}

impl Material {
    pub fn albedo(
        &self,
        atlas_tex: &Image!(2D, type=f32, sampled),
        atlas_sampler: &Sampler,
        mut hit_uv: Vec2,
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

        if self.base_color_texture == Vec4::ZERO {
            self.base_color
        } else {
            hit_uv.x = wrap(hit_uv.x);
            hit_uv.y = wrap(hit_uv.y);

            let uv = self.base_color_texture.xy()
                + hit_uv * self.base_color_texture.zw();

            self.base_color * atlas_tex.sample_by_lod(*atlas_sampler, uv, 0.0)
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
    //             images.get_unchecked(self.normal_map_texture as usize)
    //         };

    //         let normal_map_sampler = unsafe {
    //             samplers.get_unchecked(self.normal_map_texture as usize)
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
