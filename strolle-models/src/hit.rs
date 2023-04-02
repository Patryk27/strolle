use glam::{vec2, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Ray;

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub flat_normal: Vec3,
    pub uv: Vec2,
    pub material_id: u32,
}

impl Hit {
    pub const DISTANCE_OFFSET: f32 = 0.01;

    pub fn none() -> Self {
        Self {
            distance: f32::MAX,
            point: Default::default(),
            normal: Default::default(),
            flat_normal: Default::default(),
            uv: Default::default(),
            material_id: Default::default(),
        }
    }

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn from_primary(d0: Vec4, d1: Vec4, d2: Vec4, ray: Ray) -> Self {
        if d0.xyz() == Default::default() {
            Self::none()
        } else {
            Self {
                distance: Default::default(), // TODO
                point: d0.xyz() - ray.direction() * 0.001,
                normal: d1.xyz(),
                flat_normal: d2.xyz(),
                uv: vec2(d1.w, d2.w),
                material_id: d0.w.to_bits(),
            }
        }
    }
}
