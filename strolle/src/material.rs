use std::fmt::Debug;

use spirv_std::glam::{vec4, Vec4};

use crate::{gpu, Images, Params};

#[derive(Clone, Debug)]
pub struct Material<P>
where
    P: Params,
{
    base_color: Vec4,
    base_color_texture: Option<P::ImageHandle>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    refraction: f32,
    reflectivity: f32,
    normal_map_texture: Option<P::ImageHandle>,
}

impl<P> Material<P>
where
    P: Params,
{
    pub fn set_base_color(&mut self, base_color: Vec4) {
        self.base_color = base_color;
    }

    pub fn with_base_color(mut self, base_color: Vec4) -> Self {
        self.set_base_color(base_color);
        self
    }

    pub fn set_base_color_texture(
        &mut self,
        base_color_texture: Option<P::ImageHandle>,
    ) {
        self.base_color_texture = base_color_texture;
    }

    pub fn with_base_color_texture(
        mut self,
        base_color_texture: Option<P::ImageHandle>,
    ) -> Self {
        self.set_base_color_texture(base_color_texture);
        self
    }

    pub fn set_perceptual_roughness(&mut self, perceptual_roughness: f32) {
        self.perceptual_roughness = perceptual_roughness;
    }

    pub fn with_perceptual_roughness(
        mut self,
        perceptual_roughness: f32,
    ) -> Self {
        self.set_perceptual_roughness(perceptual_roughness);
        self
    }

    pub fn set_metallic(&mut self, metallic: f32) {
        self.metallic = metallic;
    }

    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.set_metallic(metallic);
        self
    }

    pub fn set_reflectance(&mut self, reflectance: f32) {
        self.reflectance = reflectance;
    }

    pub fn with_reflectance(mut self, reflectance: f32) -> Self {
        self.set_reflectance(reflectance);
        self
    }

    pub fn set_refraction(&mut self, refraction: f32) {
        self.refraction = refraction;
    }

    pub fn with_refraction(mut self, refraction: f32) -> Self {
        self.set_refraction(refraction);
        self
    }

    pub fn set_reflectivity(&mut self, reflectivity: f32) {
        self.reflectivity = reflectivity;
    }

    pub fn with_reflectivity(mut self, reflectivity: f32) -> Self {
        self.set_reflectivity(reflectivity);
        self
    }

    pub fn set_normal_map_texture(
        &mut self,
        normal_map_texture: Option<P::ImageHandle>,
    ) {
        self.normal_map_texture = normal_map_texture;
    }

    pub fn with_normal_map_texture(
        mut self,
        normal_map_texture: Option<P::ImageHandle>,
    ) -> Self {
        self.set_normal_map_texture(normal_map_texture);
        self
    }

    pub(crate) fn build(&self, images: &Images<P>) -> gpu::Material {
        gpu::Material {
            base_color: self.base_color,
            base_color_texture: images
                .lookup_opt(self.base_color_texture.as_ref())
                .unwrap_or_default(),
            perceptual_roughness: self.perceptual_roughness,
            metallic: self.metallic,
            reflectance: self.reflectance,
            refraction: self.refraction,
            reflectivity: self.reflectivity,
            normal_map_texture: images
                .lookup_opt(self.normal_map_texture.as_ref())
                .unwrap_or_default(),
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        }
    }
}

impl<P> Default for Material<P>
where
    P: Params,
{
    fn default() -> Self {
        // Defaults here more-or-less follow Bevy's `StandardMaterial` (it's not
        // a requirement though, it's just for convenience)

        Self {
            base_color: vec4(1.0, 1.0, 1.0, 1.0),
            base_color_texture: None,
            perceptual_roughness: 0.5,
            metallic: 0.0,
            reflectance: 0.5,
            refraction: 1.0,
            reflectivity: 0.0,
            normal_map_texture: None,
        }
    }
}
