use crate::{BoundingBox, BvhPrimitive};

#[derive(Debug)]
pub enum BvhNode<'a> {
    Internal {
        bounds: BoundingBox,
        left_node_id: u32,
    },

    Leaf {
        bounds: BoundingBox,
        primitives: &'a mut [BvhPrimitive],
    },
}

impl<'a> BvhNode<'a> {
    pub fn bounds(&self) -> BoundingBox {
        match self {
            BvhNode::Internal { bounds, .. } => *bounds,
            BvhNode::Leaf { bounds, .. } => *bounds,
        }
    }

    pub fn sah_cost(&self) -> f32 {
        if let BvhNode::Leaf { bounds, primitives } = self {
            (primitives.len() as f32) * bounds.half_area()
        } else {
            0.0
        }
    }
}

impl Clone for BvhNode<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Internal {
                bounds,
                left_node_id,
            } => Self::Internal {
                bounds: *bounds,
                left_node_id: *left_node_id,
            },

            Self::Leaf { .. } => {
                panic!();
            }
        }
    }
}

impl Default for BvhNode<'_> {
    fn default() -> Self {
        BvhNode::Internal {
            bounds: Default::default(),
            left_node_id: Default::default(),
        }
    }
}
