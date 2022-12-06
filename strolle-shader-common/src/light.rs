use crate::*;

pub const POINT_LIGHT: f32 = 0.0;
pub const SPOT_LIGHT: f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Light {
    // x,y,z is position, w is light type
    pos: Vec4,
    // Only applicable to spot light
    // x,y,z is position light is looking at
    // w is angle of light
    point_at: Vec4,
    // x,y,z is color
    color: Vec4,
}

impl Light {
    pub fn pos(&self) -> Vec3 {
        self.pos.truncate()
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

    pub fn is_spot(&self) -> bool {
        self.kind() == SPOT_LIGHT
    }

    pub fn is_point(&self) -> bool {
        self.kind() == POINT_LIGHT
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Light {
    pub fn point(pos: Vec3, color: Vec3) -> Self {
        Self {
            pos: pos.extend(POINT_LIGHT),
            point_at: Vec4::ZERO,
            color: color.extend(0.0),
        }
    }
}
