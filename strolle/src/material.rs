use std::fmt::Debug;

use spirv_std::glam::{vec4, Vec4};

use crate::{gpu, Images, Params};
use crate::utils::ToGpu;

#[derive(Clone, Debug)]
pub struct Material<P>
where
    P: Params,
{
    pub base_color: Vec4,
    pub base_color_texture: Option<P::ImageHandle>,
    pub emissive: Vec4,
    pub emissive_texture: Option<P::ImageHandle>,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub metallic_roughness_texture: Option<P::ImageHandle>,
    pub reflectance: f32,
    pub ior: f32,
    pub normal_map_texture: Option<P::ImageHandle>,
    pub alpha_mode: AlphaMode,
}

impl<P> Material<P>
where
    P: Params,
{
    pub(crate) fn serialize(&self, images: &Images<P>) -> gpu::Material {
        gpu::Material {
            base_color: self.base_color,
            base_color_texture: images
                .lookup_opt(self.base_color_texture)
                .unwrap_or_default().to_gpu(),
            emissive: self.emissive,
            emissive_texture: images
                .lookup_opt(self.emissive_texture)
                .unwrap_or_default().to_gpu(),
            roughness: self.perceptual_roughness.powf(2.0),
            metallic: self.metallic,
            metallic_roughness_texture: images
                .lookup_opt(self.metallic_roughness_texture)
                .unwrap_or_default().to_gpu(),
            reflectance: self.reflectance,
            ior: self.ior,
            normal_map_texture: images
                .lookup_opt(self.normal_map_texture)
                .unwrap_or_default().to_gpu(),
        }
    }
}

impl<P> Default for Material<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            base_color: vec4(1.0, 1.0, 1.0, 1.0),
            base_color_texture: None,
            emissive: Vec4::ZERO,
            emissive_texture: None,
            perceptual_roughness: 0.5,
            metallic: 0.0,
            metallic_roughness_texture: None,
            reflectance: 0.5,
            ior: 1.0,
            normal_map_texture: None,
            alpha_mode: Default::default(),
        }
    }
}

/// Specifies if a material is allowed to be transparent
#[derive(Clone, Copy, Debug, Default)]
pub enum AlphaMode {
    /// Material is always opaque (this is the default).
    ///
    /// When this is active, the base color's alpha is always set to 1.0.
    #[default]
    Opaque,

    /// Material is allowed to be transparent (i.e. base color's and base color
    /// texture's alpha channel is honored).
    ///
    /// Note that enabling this option has negative effects on ray-tracing
    /// performance (non-opaque materials need special handling during the ray
    /// traversal process), so this option should be enabled conservatively,
    /// only for materials that actually use transparency.
    Blend,
}
