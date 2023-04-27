use log::info;
use spirv_std::glam::{uvec2, Mat4, UVec2, Vec3};

use crate::gpu;

#[derive(Clone, Debug, Default)]
pub struct Camera {
    pub mode: CameraMode,
    pub viewport: CameraViewport,
    pub projection: CameraProjection,
    pub background: CameraBackground,
}

impl Camera {
    pub(crate) fn describe(&self) -> String {
        format!(
            "pos={}x{}, size={}x{}, format={:?}",
            self.viewport.position.x,
            self.viewport.position.y,
            self.viewport.size.x,
            self.viewport.size.y,
            self.viewport.format,
        )
    }

    pub(crate) fn is_invalidated_by(&self, older: &Self) -> bool {
        if self.viewport.format != older.viewport.format {
            info!(
                "Camera {} invalidated: viewport's texture format has been \
                 changed  ({:?} -> {:?})",
                older.describe(),
                older.viewport.format,
                self.viewport.format,
            );

            return true;
        }

        if self.viewport.size != older.viewport.size {
            info!(
                "Camera {} invalidated: texture format has been changed \
                 ({} -> {})",
                older.describe(),
                older.viewport.size,
                self.viewport.size,
            );

            return true;
        }

        false
    }

    pub(crate) fn serialize(&self) -> gpu::Camera {
        gpu::Camera::new(
            self.projection.projection_view,
            self.projection.origin,
            self.projection.look_at,
            self.projection.up,
            self.projection.fov,
            self.viewport.position,
            self.viewport.size,
            self.background.color,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    DisplayImage,
    DisplayDirectLightning,
    DisplayIndirectLightning,
    DisplayNormals,
}

impl CameraMode {
    pub(crate) fn serialize(&self) -> u32 {
        match self {
            CameraMode::DisplayImage => 0,
            CameraMode::DisplayDirectLightning => 1,
            CameraMode::DisplayIndirectLightning => 2,
            CameraMode::DisplayNormals => 3,
        }
    }

    pub(crate) fn needs_direct_lightning(&self) -> bool {
        matches!(
            self,
            Self::DisplayImage
                | Self::DisplayDirectLightning
                | Self::DisplayIndirectLightning
        )
    }

    pub(crate) fn needs_indirect_lightning(&self) -> bool {
        matches!(self, Self::DisplayImage | Self::DisplayIndirectLightning)
    }
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::DisplayImage
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
pub struct CameraProjection {
    pub projection_view: Mat4,
    pub origin: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
    pub fov: f32,
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
