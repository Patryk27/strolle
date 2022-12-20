use spirv_std::glam::{vec4, Vec4};
use strolle_models::TriangleId;

use super::*;

pub struct BvhSerializer<'a> {
    root: Option<&'a BvhNode>,
}

impl<'a> BvhSerializer<'a> {
    const OP_NODE: u32 = 0;
    const OP_LEAF: u32 = 1;

    pub fn new(root: Option<&'a BvhNode>) -> Self {
        Self { root }
    }

    pub fn serialize_into(self, out: &mut Vec<Vec4>) {
        if let Some(root) = self.root {
            Self::process(out, root);
        } else {
            Self::process(
                out,
                &BvhNode::Leaf {
                    bb: BoundingBox::default().with(Default::default()),
                    tri: TriangleId::new(0),
                },
            );
        }
    }

    fn process(out: &mut Vec<Vec4>, node: &BvhNode) -> usize {
        let ptr = out.len();

        out.push(Default::default());
        out.push(Default::default());

        match node {
            BvhNode::Node { bb, left, right } => {
                let _left_ptr = Self::process(out, left);
                let right_ptr = Self::process(out, right);

                let meta = f32::from_bits({
                    let payload = right_ptr as u32;

                    Self::OP_NODE | (payload << 1)
                });

                out[ptr] = vec4(meta, bb.min().x, bb.min().y, bb.min().z);
                out[ptr + 1] = bb.max().extend(0.0);
            }

            BvhNode::Leaf { bb, tri } => {
                let meta = f32::from_bits({
                    let payload = tri.get() as u32;

                    Self::OP_LEAF | (payload << 1)
                });

                out[ptr] = vec4(meta, bb.min().x, bb.min().y, bb.min().z);
                out[ptr + 1] = bb.max().extend(0.0);
            }
        }

        ptr
    }
}
