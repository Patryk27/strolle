use glam::{Vec2, Vec3};

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub uv: Vec2,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Hit {
    const MAX_DIST: f32 = f32::MAX;

    pub fn none() -> Self {
        Self {
            distance: Self::MAX_DIST,
            uv: Default::default(),
            point: Default::default(),
            normal: Default::default(),
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
