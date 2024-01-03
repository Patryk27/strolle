use std::fmt::Debug;

use bevy::asset::AssetId;
use bevy::pbr::StandardMaterial;

// pub(crate) fn serialize(&self, images: &Images<P>) -> gpu::Material {
//     gpu::Material {
//         base_color: self.base_color,
//         base_color_texture: images
//             .lookup_opt(self.base_color_texture.as_ref())
//             .unwrap_or_default(),
//         emissive: self.emissive,
//         emissive_texture: images
//             .lookup_opt(self.emissive_texture.as_ref())
//             .unwrap_or_default(),
//         roughness: self.perceptual_roughness.powf(2.0),
//         metallic: self.metallic,
//         reflectance: self.reflectance,
//         ior: self.ior,
//         normal_map_texture: images
//             .lookup_opt(self.normal_map_texture.as_ref())
//             .unwrap_or_default(),
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialHandle(AssetId<StandardMaterial>);

impl MaterialHandle {
    pub fn new(asset: AssetId<StandardMaterial>) -> Self {
        Self(asset)
    }
}
