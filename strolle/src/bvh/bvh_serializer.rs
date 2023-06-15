use spirv_std::glam::{vec4, Vec4};

use super::*;

pub struct BvhSerializer;

impl BvhSerializer {
    const OP_INTERNAL: u32 = 0;
    const OP_LEAF: u32 = 1;

    pub fn process(out: &mut Vec<Vec4>, node: &BvhNode) -> usize {
        let idx = out.len();

        match node {
            BvhNode::Internal {
                bb, left, right, ..
            } => {
                out.push(Default::default());
                out.push(Default::default());

                let _left_ptr = Self::process(out, left);
                let right_ptr = Self::process(out, right);

                let opcode = Self::OP_INTERNAL;
                let arg0 = right_ptr as u32;
                let arg1 = 0;

                out[idx] = vec4(
                    f32::from_bits(opcode | (arg0 << 1)),
                    bb.min().x,
                    bb.min().y,
                    bb.min().z,
                );

                out[idx + 1] = bb.max().extend(f32::from_bits(arg1));
            }

            BvhNode::Leaf { bb, tris } => {
                for (tri_idx, (tri_id, mat_id)) in tris.iter().enumerate() {
                    let opcode = Self::OP_LEAF;

                    let arg0 = if tri_idx + 1 == tris.len() {
                        tri_id.get() << 1
                    } else {
                        (tri_id.get() << 1) | 1
                    };

                    let arg1 = mat_id.get();

                    out.push(vec4(
                        f32::from_bits(opcode | (arg0 << 1)),
                        bb.min().x,
                        bb.min().y,
                        bb.min().z,
                    ));

                    out.push(bb.max().extend(f32::from_bits(arg1)));
                }
            }
        }

        idx / 2
    }
}
