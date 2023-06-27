use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::AsBindGroup;
use strolle as st;

use crate::utils::{color_to_vec4, GlamCompat};
use crate::EngineParams;

/// Extends Bevy's `StandardMaterial` with extra features supported by Strolle.
#[derive(Clone, Debug, TypeUuid, AsBindGroup)]
#[uuid = "b270a5e8-9330-11ed-a1eb-0242ac120002"]
pub struct StrolleMaterial {
    pub parent: StandardMaterial,

    /// Specifies the index of refraction.
    ///
    /// Defaults to 1.0 and makes sense only for transparent materials (i.e.
    /// when `parent.base_color` and/or `parent.base_color_texture` have
    /// transparency, and `parent.alpha_mode` is non-opaque).
    pub refraction: f32,

    /// Specifies the reflectivity level (0.0 ..= 1.0).
    ///
    /// Defaults to 0.0, making the material non-reflective, while the value of
    /// 1.0 means the material will behave as mirror.
    ///
    /// Note that it's different from `parent.reflectance` in the sense that
    /// reflectance only applies to the specular intensity (i.e. how much
    /// _lights_ are reflected), while setting reflectivity actually causes the
    /// material to reflect the rays.
    pub reflectivity: f32,
}

impl Default for StrolleMaterial {
    fn default() -> Self {
        Self {
            parent: Default::default(),
            refraction: 1.0,
            reflectivity: 0.0,
        }
    }
}

impl Material for StrolleMaterial {
    //
}

pub(crate) trait MaterialLike
where
    Self: TypeUuid + Clone + Send + Sync + 'static,
{
    fn into_material(self) -> st::Material<EngineParams>;
    fn map_handle(handle: Handle<Self>) -> MaterialHandle;
    fn images(&self) -> Vec<&Handle<Image>>;
}

impl MaterialLike for StandardMaterial {
    fn into_material(self) -> st::Material<EngineParams> {
        let base_color = {
            let color = color_to_vec4(self.base_color);

            match self.alpha_mode {
                AlphaMode::Opaque => color.xyz().extend(1.0),
                AlphaMode::Mask(mask) => {
                    if color.w >= mask {
                        color.xyz().extend(1.0)
                    } else {
                        color.xyz().extend(0.0)
                    }
                }
                _ => color,
            }
        };

        st::Material::default()
            .with_base_color(base_color.compat())
            .with_base_color_texture(self.base_color_texture)
            .with_emissive(color_to_vec4(self.emissive).compat())
            .with_emissive_texture(self.emissive_texture)
            .with_perceptual_roughness(self.perceptual_roughness)
            .with_metallic(self.metallic)
            .with_reflectance(self.reflectance)
            .with_normal_map_texture(self.normal_map_texture)
    }

    fn map_handle(handle: Handle<Self>) -> MaterialHandle {
        MaterialHandle::StandardMaterial(handle)
    }

    fn images(&self) -> Vec<&Handle<Image>> {
        self.base_color_texture
            .as_ref()
            .into_iter()
            .chain(self.normal_map_texture.as_ref())
            .collect()
    }
}

impl MaterialLike for StrolleMaterial {
    fn into_material(self) -> st::Material<EngineParams> {
        self.parent
            .into_material()
            .with_refraction(self.refraction)
            .with_reflectivity(self.reflectivity)
    }

    fn map_handle(handle: Handle<Self>) -> MaterialHandle {
        MaterialHandle::StrolleMaterial(handle)
    }

    fn images(&self) -> Vec<&Handle<Image>> {
        self.parent.images()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MaterialHandle {
    StandardMaterial(Handle<StandardMaterial>),
    StrolleMaterial(Handle<StrolleMaterial>),
}
