use std::hash::{Hash, Hasher};
use std::ops::Range;

use glam::Vec3;

use crate::bvh::BvhNodeId;
use crate::gpu;
use crate::utils::BoundingBox;

#[derive(Clone, Copy, Debug)]
pub enum Primitive {
    Triangle {
        center: Vec3,
        bounds: BoundingBox,
        triangle_id: gpu::TriangleId,
        material_id: gpu::MaterialId,
    },

    Instance {
        center: Vec3,
        bounds: BoundingBox,
        node_id: BvhNodeId,
    },

    Killed,
}

impl Primitive {
    pub fn center(&self) -> Vec3 {
        match self {
            Primitive::Triangle { center, .. }
            | Primitive::Instance { center, .. } => *center,

            Primitive::Killed => Default::default(),
        }
    }

    pub fn bounds(&self) -> BoundingBox {
        match self {
            Primitive::Triangle { bounds, .. }
            | Primitive::Instance { bounds, .. } => *bounds,

            Primitive::Killed => Default::default(),
        }
    }
}

impl Hash for Primitive {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        // match self {
        //     Primitive::Triangle { hash, .. }
        //     | Primitive::Instance { hash, .. } => {
        //         hash.hash(state);
        //     }

        //     Primitive::Killed => {
        //         //
        //     }
        // }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveId(u32);

impl PrimitiveId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrimitivesRef {
    start: PrimitiveId,
    end: PrimitiveId,
}

impl PrimitivesRef {
    pub fn single(id: PrimitiveId) -> Self {
        Self::range(id, PrimitiveId::new(id.get() + 1))
    }

    pub fn range(start: PrimitiveId, end: PrimitiveId) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> PrimitiveId {
        self.start
    }

    pub fn end(&self) -> PrimitiveId {
        self.end
    }

    pub fn offset(&mut self, offset: i32) {
        self.start =
            PrimitiveId::new((self.start.get() as i32 + offset) as u32);

        self.end = PrimitiveId::new((self.end.get() as i32 + offset) as u32);
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

impl Default for PrimitivesRef {
    fn default() -> Self {
        Self::range(PrimitiveId::new(0), PrimitiveId::new(0))
    }
}
