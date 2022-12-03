use crate::*;

#[derive(Copy, Clone)]
pub struct Hit {
    pub t: f32,
    pub uv: Vec4,
    pub ray: Ray,
    pub point: Vec3,
    pub normal: Vec3,
    pub tri_id: TriangleId<AnyTriangle>,
    pub mat_id: MaterialId,
    pub alpha: f32,
}

impl Hit {
    const MAX_T: f32 = 1000.0;

    pub fn none() -> Self {
        Self {
            t: Self::MAX_T,
            uv: Default::default(),
            ray: Default::default(),
            point: Default::default(),
            normal: Default::default(),
            tri_id: TriangleId::new(AnyTriangle, 0),
            mat_id: MaterialId::new(0),
            alpha: 1.0,
        }
    }

    pub fn is_some(&self) -> bool {
        self.t < Self::MAX_T
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn is_closer_than(&self, other: Self) -> bool {
        self.t < other.t
    }
}
