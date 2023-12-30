use crate::{BoundingBox, PrimitivesRef};

#[derive(Clone, Copy, Debug)]
pub enum BvhNode {
    Internal {
        bounds: BoundingBox,
        primitives_ref: PrimitivesRef,
        left_id: BvhNodeId,
        left_hash: BvhNodeHash,
        right_id: BvhNodeId,
        right_hash: BvhNodeHash,
    },

    Leaf {
        bounds: BoundingBox,
        primitives_ref: PrimitivesRef,
    },
}

impl BvhNode {
    pub fn bounds(&self) -> BoundingBox {
        match self {
            BvhNode::Internal { bounds, .. } => *bounds,
            BvhNode::Leaf { bounds, .. } => *bounds,
        }
    }

    pub fn primitives_ref(&self) -> PrimitivesRef {
        match self {
            BvhNode::Internal { primitives_ref, .. } => *primitives_ref,
            BvhNode::Leaf { primitives_ref, .. } => *primitives_ref,
        }
    }

    pub fn sah_cost(&self) -> f32 {
        if let BvhNode::Leaf {
            bounds,
            primitives_ref,
        } = self
        {
            (primitives_ref.len() as f32) * bounds.half_area()
        } else {
            0.0
        }
    }
}

impl Default for BvhNode {
    fn default() -> Self {
        BvhNode::Leaf {
            bounds: Default::default(),
            primitives_ref: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhNodeId(u32);

impl BvhNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn root() -> Self {
        Self::new(0)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhNodeHash(u64);

impl BvhNodeHash {
    pub fn new(hash: u64) -> Self {
        Self(hash)
    }
}
