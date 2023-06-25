use super::*;
use crate::gpu;

#[derive(Clone, Debug)]
pub enum BvhNode {
    Internal {
        bb: BoundingBox,
        left: Box<Self>,
        left_hash: u64,
        right: Box<Self>,
        right_hash: u64,
    },

    Leaf {
        bb: BoundingBox,
        triangles: Vec<(gpu::TriangleId, gpu::MaterialId)>,
    },
}
