use strolle_raytracer_models::TriangleId;

use super::*;

#[derive(Clone, Debug)]
pub enum BvhNode {
    Leaf {
        tris: Vec<TriangleId>,
    },

    Node {
        bb: BoundingBox,
        left: Box<Self>,
        right: Box<Self>,
    },
}
