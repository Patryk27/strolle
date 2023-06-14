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
        let (onb_u, onb_v, onb_w) = gpu::OrthonormalBasis::build(
            self.projection.origin,
            self.projection.look_at,
            self.projection.up,
        );

        gpu::Camera {
            projection_view: self.projection.projection_view,
            origin: self.projection.origin.extend(self.projection.fov),
            viewport: self
                .viewport
                .position
                .as_vec2()
                .extend(self.viewport.size.x as f32)
                .extend(self.viewport.size.y as f32),
            onb_u,
            onb_v,
            onb_w,
            clear_color: self.background.color.extend(0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    /// The default mode - shows the final image
    Image,

    /// Shows only direct lightning
    DirectLightning,

    /// Shows only indirect lightning
    IndirectLightning,

    /// Shows only normals
    Normals,

    /// Shows a heatmap of the BVH tree
    BvhHeatmap,
}

impl CameraMode {
    pub(crate) fn serialize(&self) -> u32 {
        match self {
            CameraMode::Image => 0,
            CameraMode::DirectLightning => 1,
            CameraMode::IndirectLightning => 2,
            CameraMode::Normals => 3,
            CameraMode::BvhHeatmap => 4,
        }
    }

    pub(crate) fn needs_indirect_lightning(&self) -> bool {
        matches!(self, Self::Image | Self::IndirectLightning)
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
