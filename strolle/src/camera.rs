use std::fmt;

use glam::vec4;
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
        if self.viewport.format != older.viewport.format {
            info!(
                "Camera `{}` invalidated: viewport's texture format has been \
                 changed  ({:?} -> {:?})",
                older, older.viewport.format, self.viewport.format,
            );

            return true;
        }

        if self.viewport.size != older.viewport.size {
            info!(
                "Camera `{}` invalidated: texture format has been changed \
                 ({} -> {})",
                older, older.viewport.size, self.viewport.size,
            );

            return true;
        }

        false
    }

    pub(crate) fn serialize(&self) -> gpu::Camera {
        let t = if let CameraMode::Reference { depth } = self.mode {
            f32::from_bits(depth as u32)
        } else {
            0.0
        };

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
            data: vec4(
                f32::from_bits(self.mode.serialize()),
                t,
                Default::default(),
                Default::default(),
            ),
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

    /// Shows a path-traced reference image; slow
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
