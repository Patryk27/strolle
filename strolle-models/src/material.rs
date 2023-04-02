use bytemuck::{Pod, Zeroable};
#[cfg(not(target_arch = "spirv"))]
use glam::vec4;
use glam::Vec4;
use spirv_std::{Image, Sampler};

use crate::{Hit, MAX_IMAGES};

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    pub base_color: Vec4,
    pub base_color_texture: u32,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub refraction: f32,
    pub reflectivity: f32,
    _pad1: f32,
    _pad2: f32,
}

impl Material {
    pub fn albedo(
        &self,
        images: &[Image!(2D, type=f32, sampled); MAX_IMAGES],
        samplers: &[Sampler; MAX_IMAGES],
        hit: Hit,
    ) -> Vec4 {
        if self.base_color_texture == u32::MAX {
            self.base_color
        } else {
            let image = unsafe {
                images.get_unchecked(self.base_color_texture as usize)
            };

            let sampler = unsafe {
                samplers.get_unchecked(self.base_color_texture as usize)
            };

            self.base_color * image.sample_by_lod(*sampler, hit.uv, 0.0)
        }
    }
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
