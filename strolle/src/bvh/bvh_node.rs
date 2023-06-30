use super::*;

#[derive(Debug)]
pub enum BvhNode<'a> {
    Internal {
        bb: BoundingBox,
        left_node_id: u32,
    },

    Leaf {
        bb: BoundingBox,
        triangles: &'a mut [BvhTriangle],
    },
}

impl<'a> BvhNode<'a> {
    pub fn sah_cost(&self) -> f32 {
        if let BvhNode::Leaf { bb, triangles } = self {
            (triangles.len() as f32) * bb.area()
        } else {
            0.0
        }
    }
}

impl Clone for BvhNode<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Internal { bb, left_node_id } => Self::Internal {
                bb: bb.clone(),
                left_node_id: left_node_id.clone(),
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
            bb: Default::default(),
            left_node_id: Default::default(),
        }
    }
}
