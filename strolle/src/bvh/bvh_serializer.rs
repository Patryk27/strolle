use spirv_std::glam::{vec4, Vec4};

use super::*;

pub struct BvhSerializer;

impl BvhSerializer {
    const OP_NODE: u32 = 0;
    const OP_LEAF: u32 = 1;
    const ARG_LEAF_INSTANCE: u32 = 0;
    const ARG_LEAF_TRIANGLE: u32 = 1;

    pub fn process(out: &mut Vec<Vec4>, node: &BvhNode) -> usize {
        let ptr = out.len();

        out.push(Default::default());
        out.push(Default::default());

        match node {
            BvhNode::Internal { bb, left, right } => {
                let left_size = Self::process(out, left);
                let _right_size = Self::process(out, right);

                let opcode = f32::from_bits({
                    let payload = left_size as u32;

                    Self::OP_NODE | (payload << 1)
                });

                out[ptr] = vec4(opcode, bb.min().x, bb.min().y, bb.min().z);
                out[ptr + 1] = bb.max().extend(Default::default());
            }

            BvhNode::Leaf { bb, payload } => {
                let opcode = f32::from_bits({
                    let opcode = match payload {
                        BvhNodePayload::Instance(payload) => {
                            Self::ARG_LEAF_INSTANCE | (payload.get() << 1)
                        }
                        BvhNodePayload::Triangle(payload) => {
                            Self::ARG_LEAF_TRIANGLE | (payload.get() << 1)
                        }
                    };

                    Self::OP_LEAF | (opcode << 1)
                });

                out[ptr] = vec4(opcode, bb.min().x, bb.min().y, bb.min().z);
                out[ptr + 1] = bb.max().extend(Default::default());
            }
        }

        // We're returning sizes instead of pointers, because we keep our BVHs
        // (mostly) relatively-indexed - thanks to this approach, we don't have
        // to rebuild mesh-bvhs (unless the mesh actually changes)
        out.len() - ptr
    }
}
