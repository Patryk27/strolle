use std::fmt;

use strolle_models as gpu;

use super::bvh_printer::BvhPrinter;
use super::*;

#[derive(Clone, Debug)]
pub enum BvhNode {
    Internal {
        bb: BoundingBox,
        left: Box<Self>,
        right: Box<Self>,
    },

    Leaf {
        bb: BoundingBox,
        triangle_id: gpu::TriangleId,
        material_id: gpu::MaterialId,
    },
}

impl BvhNode {
    pub fn validate(&self) {
        if let BvhNode::Internal { bb, left, right } = self {
            left.validate_assert(*bb);
            left.validate();

            right.validate_assert(*bb);
            right.validate();
        }
    }

    fn validate_assert(&self, parent_bb: BoundingBox) {
        let bb = self.bb();

        assert!(bb.min().x >= parent_bb.min().x);
        assert!(bb.min().y >= parent_bb.min().y);
        assert!(bb.min().z >= parent_bb.min().z);

        assert!(bb.max().x <= parent_bb.max().x);
        assert!(bb.max().y <= parent_bb.max().y);
        assert!(bb.max().z <= parent_bb.max().z);
    }

    pub fn bb(&self) -> BoundingBox {
        match self {
            BvhNode::Internal { bb, .. } => *bb,
            BvhNode::Leaf { bb, .. } => *bb,
        }
    }
}

impl fmt::Display for BvhNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BvhPrinter::print(self))
    }
}
