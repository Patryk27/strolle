use glam::Vec3;

use crate::gpu;

#[derive(Clone, Debug)]
pub struct Light {
    position: Vec3,
    radius: f32,
    color: Vec3,
    range: f32,
}

impl Light {
    pub fn point(position: Vec3, radius: f32, color: Vec3, range: f32) -> Self {
        Self {
            position,
            radius,
            color,
            range,
        }
    }

    pub(crate) fn build(&self) -> gpu::Light {
        gpu::Light {
            d0: self.position.extend(self.radius),
            d1: self.color.extend(self.range),
        }
    }
}
