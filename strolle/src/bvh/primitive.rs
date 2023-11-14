use std::hash::{Hash, Hasher};
use std::ops::Range;

use glam::Vec3;

use crate::gpu;
use crate::utils::BoundingBox;

#[derive(Clone, Copy, Debug)]
pub struct BvhPrimitive {
    pub triangle_id: gpu::TriangleId,
    pub material_id: gpu::MaterialId,
    pub center: Vec3,
    pub bounds: BoundingBox,
    pub updated_at: u32,
}

impl BvhPrimitive {
    pub fn kill(&mut self) {
        self.center = Vec3::MAX;
    }

    pub fn is_alive(&self) -> bool {
        self.center.x != f32::MAX
    }
}

impl Hash for BvhPrimitive {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.center.x.to_bits().hash(state);
        self.center.y.to_bits().hash(state);
        self.center.z.to_bits().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhPrimitiveId(u32);

impl BvhPrimitiveId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BvhPrimitivesRef {
    start: BvhPrimitiveId,
    end: BvhPrimitiveId,
}

impl BvhPrimitivesRef {
    pub fn new(start: BvhPrimitiveId, end: BvhPrimitiveId) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> BvhPrimitiveId {
        self.start
    }

    pub fn end(&self) -> BvhPrimitiveId {
        self.end
    }

    pub fn offset(&mut self, offset: i32) {
        self.start =
            BvhPrimitiveId::new((self.start.get() as i32 + offset) as u32);

        self.end = BvhPrimitiveId::new((self.end.get() as i32 + offset) as u32);
    }

    pub fn as_range(&self) -> Range<usize> {
        let start = self.start.get() as usize;
        let end = self.end.get() as usize;

        start..end
    }

    pub fn len(&self) -> usize {
        (self.end.get() - self.start.get()) as usize
    }
}

impl Default for BvhPrimitivesRef {
    fn default() -> Self {
        Self::new(BvhPrimitiveId::new(0), BvhPrimitiveId::new(0))
    }
}
