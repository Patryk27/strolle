use std::fmt;

use log::info;
use spirv_std::glam::{uvec2, Mat4, UVec2, Vec3};

use crate::gpu;

#[derive(Clone, Debug, Default)]
pub struct Camera {
    pub mode: CameraMode,
    pub viewport: CameraViewport,
    pub transform: Mat4,
    pub projection: Mat4,
}

impl Camera {
    pub(crate) fn is_invalidated_by(&self, older: &Self) -> bool {
        if self.mode != older.mode {
            info!(
                "Camera `{}` invalidated: mode has been changed ({:?} -> {:?})",
                older, older.mode, self.mode,
            );

            return true;
        }

        if self.viewport.format != older.viewport.format {
            info!(
                "Camera `{}` invalidated: viewport.format has been changed \
                 ({:?} -> {:?})",
                older, older.viewport.format, self.viewport.format,
            );

            return true;
        }

        if self.viewport.size != older.viewport.size {
            info!(
                "Camera `{}` invalidated: viewport.size has been changed \
                 ({} -> {})",
                older, older.viewport.size, self.viewport.size,
            );

            return true;
        }

        false
    }

    pub(crate) fn serialize(&self) -> gpu::Camera {
        gpu::Camera {
            projection_view: self.projection * self.transform.inverse(),
            ndc_to_world: self.transform * self.projection.inverse(),
            origin: self
                .transform
                .to_scale_rotation_translation()
                .2
                .extend(Default::default()),
            screen: self
                .viewport
                .size
                .as_vec2()
                .extend(Default::default())
                .extend(Default::default()),
        }
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pos={}x{}, size={}x{}, format={:?}",
            self.viewport.position.x,
            self.viewport.position.y,
            self.viewport.size.x,
            self.viewport.size.y,
            self.viewport.format,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    /// Shows the final composed image, default
    Image { denoise: bool },

    /// Shows direct diffuse lighting
    DiDiffuse { denoise: bool },

    /// Shows direct specular lighting
    DiSpecular { denoise: bool },

    /// Shows indirect diffuse lighting
    GiDiffuse { denoise: bool },

    /// Shows indirect specular lighting
    GiSpecular { denoise: bool },

    /// Shows BVH tree's heatmap
    BvhHeatmap,

    /// Shows a path-traced reference image; slow
    Reference { depth: u8 },
}

impl CameraMode {
    pub(crate) fn serialize(&self) -> u32 {
        match self {
            CameraMode::Image { .. } => 0,
            CameraMode::DiDiffuse { .. } => 1,
            CameraMode::DiSpecular { .. } => 2,
            CameraMode::GiDiffuse { .. } => 3,
            CameraMode::GiSpecular { .. } => 4,
            CameraMode::BvhHeatmap => 5,
            CameraMode::Reference { .. } => 6,
        }
    }

    pub(crate) fn needs_di(&self) -> bool {
        matches!(
            self,
            Self::Image { .. }
                | Self::DiDiffuse { .. }
                | Self::DiSpecular { .. }
        )
    }

    pub(crate) fn needs_gi(&self) -> bool {
        matches!(
            self,
            Self::Image { .. }
                | Self::GiDiffuse { .. }
                | Self::GiSpecular { .. }
        )
    }

    pub(crate) fn denoise(&self) -> bool {
        matches!(
            self,
            Self::Image { denoise: true }
                | Self::DiDiffuse { denoise: true }
                | Self::DiSpecular { denoise: true }
                | Self::GiDiffuse { denoise: true }
                | Self::GiSpecular { denoise: true }
        )
    }

    pub(crate) fn denoise_di_diff(&self) -> bool {
        matches!(
            self,
            Self::Image { denoise: true } | Self::DiDiffuse { denoise: true }
        )
    }

    pub(crate) fn denoise_gi_diff(&self) -> bool {
        matches!(
            self,
            Self::Image { denoise: true } | Self::GiDiffuse { denoise: true }
        )
    }
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Image { denoise: true }
    }
}

#[derive(Clone, Debug)]
pub struct CameraViewport {
    pub format: wgpu::TextureFormat,
    pub size: UVec2,
    pub position: UVec2,
}

impl Default for CameraViewport {
    fn default() -> Self {
        Self {
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            size: uvec2(512, 512),
            position: uvec2(0, 0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CameraBackground {
    pub color: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CameraHandle(usize);

impl CameraHandle {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}
