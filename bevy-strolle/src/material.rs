use bevy::math::Vec4Swizzles;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::AsBindGroup;
use strolle as st;

use crate::utils::color_to_vec4;
use crate::EngineParams;

/// Extends Bevy's `StandardMaterial` with extra features supported by Strolle.
#[derive(Clone, Debug, Default, TypeUuid, AsBindGroup)]
#[uuid = "b270a5e8-9330-11ed-a1eb-0242ac120002"]
pub struct StrolleMaterial {
    pub parent: StandardMaterial,
    pub refraction: f32,
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
                AlphaMode::Blend => color,
            }
        };

        st::Material::default()
            .with_base_color(base_color)
            .with_base_color_texture(self.base_color_texture)
            .with_perceptual_roughness(self.perceptual_roughness)
            .with_metallic(self.metallic)
            .with_reflectance(self.reflectance)
    }

    fn map_handle(handle: Handle<Self>) -> MaterialHandle {
        MaterialHandle::StandardMaterial(handle)
    }
}

impl MaterialLike for StrolleMaterial {
    fn into_material(self) -> st::Material<EngineParams> {
        self.parent.into_material().with_refraction(self.refraction)
    }

    fn map_handle(handle: Handle<Self>) -> MaterialHandle {
        MaterialHandle::StrolleMaterial(handle)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MaterialHandle {
    StandardMaterial(Handle<StandardMaterial>),
    StrolleMaterial(Handle<StrolleMaterial>),
}
