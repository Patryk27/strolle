use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, Vec4Swizzles};

use crate::Noise;

pub const POINT_LIGHT: f32 = 0.0;
pub const SPOT_LIGHT: f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Light {
    // x,y,z is position, w is light type
    pos: Vec4,
    // Only applicable to spot light
    // x,y,z is position light is looking at
    // w is angle of light
    point_at: Vec4,
    // x,y,z is color
    // w is range
    color: Vec4,
}

impl Light {
    pub fn center(&self) -> Vec3 {
        self.pos.truncate()
    }

    pub fn position(&self, _noise: &mut Noise) -> Vec3 {
        self.center()
        // self.center() + self.radius() * noise.sample_sphere() TODO
    }

    pub fn point_at(&self) -> Vec3 {
        self.point_at.xyz()
    }

    pub fn cone_angle(&self) -> f32 {
        self.point_at.w
    }

    pub fn kind(&self) -> f32 {
        self.pos.w
    }

    pub fn color(&self) -> Vec3 {
        self.color.truncate()
    }

    pub fn range(&self) -> f32 {
        self.color.w
    }

    // TODO: Make configurable
    pub fn radius(&self) -> f32 {
        0.1
    }

    pub fn is_spot(&self) -> bool {
        self.kind() == SPOT_LIGHT
    }

    pub fn is_point(&self) -> bool {
        self.kind() == POINT_LIGHT
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Light {
    pub fn point(pos: Vec3, color: Vec3, range: f32) -> Self {
        Self {
            pos: pos.extend(POINT_LIGHT),
            point_at: Vec4::ZERO,
            color: color.extend(range),
        }
    }
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct LightId(u32);

impl LightId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }
}
