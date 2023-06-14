use glam::Vec3;

use crate::gpu;

#[derive(Clone, Debug)]
pub struct Light {
    position: Vec3,
    color: Vec3,
    range: f32,
}

impl Light {
    pub fn point(position: Vec3, color: Vec3, range: f32) -> Self {
        Self {
            position,
            color,
            range,
        }
    }

    pub(crate) fn build(&self) -> gpu::Light {
        gpu::Light {
            d0: self.position.extend(Default::default()),
            d1: self.color.extend(self.range),
        }
    }
}
