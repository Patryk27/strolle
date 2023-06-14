use spirv_std::glam::{vec4, Vec4};

use super::*;

pub struct BvhSerializer;

impl BvhSerializer {
    const OP_INTERNAL: u32 = 0;
    const OP_LEAF: u32 = 1;

    pub fn process(out: &mut Vec<Vec4>, node: &BvhNode) -> usize {
        let ptr = out.len();

        out.push(Default::default());
        out.push(Default::default());

        match node {
            BvhNode::Internal {
                bb, left, right, ..
            } => {
                let left_ptr = Self::process(out, left);
                let _right_ptr = Self::process(out, right);

                let opcode = Self::OP_INTERNAL;
                let arg0 = left_ptr as u32;
                let arg1 = 0;

                out[ptr] = vec4(
                    f32::from_bits(opcode | (arg0 << 1)),
                    bb.min().x,
                    bb.min().y,
                    bb.min().z,
                );

                out[ptr + 1] = bb.max().extend(f32::from_bits(arg1));
            }

            BvhNode::Leaf {
                bb,
                triangle_id,
                material_id,
            } => {
                let opcode = Self::OP_LEAF;
                let arg0 = triangle_id.get();
                let arg1 = material_id.get();

                out[ptr] = vec4(
                    f32::from_bits(opcode | (arg0 << 1)),
                    bb.min().x,
                    bb.min().y,
                    bb.min().z,
                );

                out[ptr + 1] = bb.max().extend(f32::from_bits(arg1));
            }
        }

        out.len()
    }
}
