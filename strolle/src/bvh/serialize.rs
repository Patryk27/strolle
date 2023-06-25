use spirv_std::glam::{vec4, Vec4};

use super::*;

pub fn run(out: &mut Vec<Vec4>, node: &BvhNode) -> u32 {
    const OP_INTERNAL: u32 = 0;
    const OP_LEAF: u32 = 1;

    let ptr = out.len();

    match node {
        BvhNode::Internal {
            bb, left, right, ..
        } => {
            out.push(Default::default());
            out.push(Default::default());

            let _left_ptr = run(out, left);
            let right_ptr = run(out, right);

            out[ptr] = vec4(
                bb.min().x,
                bb.min().y,
                bb.min().z,
                f32::from_bits(OP_INTERNAL | (right_ptr << 1)),
            );

            out[ptr + 1] =
                vec4(bb.max().x, bb.max().y, bb.max().z, Default::default());
        }

        BvhNode::Leaf { bb, triangles } => {
            for (triangle_idx, (triangle_id, material_id)) in
                triangles.iter().enumerate()
            {
                let arg0 = triangle_id.get() << 1;

                // If there are more triangles following this one, toggle the
                // first bit
                let arg0 = if triangle_idx + 1 == triangles.len() {
                    arg0
                } else {
                    arg0 | 1
                };

                let arg1 = material_id.get();

                out.push(vec4(
                    bb.min().x,
                    bb.min().y,
                    bb.min().z,
                    f32::from_bits(OP_LEAF | (arg0 << 1)),
                ));

                out.push(vec4(
                    bb.max().x,
                    bb.max().y,
                    bb.max().z,
                    f32::from_bits(arg1),
                ));
            }
        }
    }

    // In the shader we use `BvhNode` that takes 2 * Vec4 of space and so here
    // we divide by two in order to compensate for that
    (ptr / 2) as u32
}
