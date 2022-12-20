use crate::*;

#[derive(Copy, Clone)]
pub struct Hit {
    pub dist: f32,
    pub uv: Vec2,
    pub ray: Ray,
    pub point: Vec3,
    pub normal: Vec3,
    pub mat_id: MaterialId,
}

impl Hit {
    const MAX_DIST: f32 = f32::MAX;

    pub fn none() -> Self {
        Self {
            dist: Self::MAX_DIST,
            uv: Default::default(),
            ray: Default::default(),
            point: Default::default(),
            normal: Default::default(),
            mat_id: MaterialId::new(0),
        }
    }

    pub fn is_some(&self) -> bool {
        self.dist < Self::MAX_DIST
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_closer_than(&self, other: Self) -> bool {
        self.dist < other.dist
    }
}
