use bevy::prelude::Component;

/// Extends Bevy's camera with extra features supported by Strolle.
///
/// This is a component that can be attached into Bevy's `Camera`; when not
/// attached, the default configuration is used.
#[derive(Clone, Debug, Default, Component)]
pub struct StrolleCamera {
    pub mode: CameraMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    /// Default mode - shows the final image
    Image,

    /// Shows direct lightning
    DirectLightning,

    /// Shows indirect diffuse lightning
    IndirectDiffuseLightning,

    /// Shows indirect diffuse lightning
    IndirectSpecularLightning,

    /// Shows BVH tree's heatmap
    BvhHeatmap,

    /// Shows a path-traced reference image
    Reference { depth: u8 },
}

impl CameraMode {
    pub(crate) fn serialize(&self) -> u32 {
        match self {
            CameraMode::Image => 0,
            CameraMode::DirectLightning => 1,
            CameraMode::IndirectDiffuseLightning => 2,
            CameraMode::IndirectSpecularLightning => 3,
            CameraMode::BvhHeatmap => 4,
            CameraMode::Reference { .. } => 5,
        }
    }

    pub(crate) fn needs_direct_lightning(&self) -> bool {
        matches!(self, Self::Image | Self::DirectLightning)
    }

    pub(crate) fn needs_indirect_diffuse_lightning(&self) -> bool {
        matches!(self, Self::Image | Self::IndirectDiffuseLightning)
    }

    pub(crate) fn needs_indirect_specular_lightning(&self) -> bool {
        matches!(self, Self::Image | Self::IndirectSpecularLightning)
    }
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Image
    }
}
