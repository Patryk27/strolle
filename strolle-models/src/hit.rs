use glam::{Vec2, Vec3};

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub texture_uv: Vec2,
}

impl Hit {
    const MAX_DIST: f32 = f32::MAX;

    pub fn none() -> Self {
        Self {
            distance: Self::MAX_DIST,
            point: Default::default(),
            normal: Default::default(),
            texture_uv: Default::default(),
        }
    }

    pub fn is_some(&self) -> bool {
        self.distance < Self::MAX_DIST
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_closer_than(&self, other: Self) -> bool {
        self.distance < other.distance
    }
}
