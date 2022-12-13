use spirv_std::glam::{vec4, Vec4};

use super::*;

pub fn serialize(data: &mut Vec<Vec4>, rbvh: RopedBvh) {
    data.clear();

    for node in rbvh {
        let v1;
        let v2;

        match node {
            RopedBvhNode::Leaf { triangle, goto_id } => {
                let goto_ptr = goto_id.map(|id| id * 2).unwrap_or_default();

                let info = 1
                    | ((triangle.get() as u32) << 1)
                    | ((goto_ptr as u32) << 16);

                v1 = vec4(0.0, 0.0, 0.0, f32::from_bits(info));
                v2 = vec4(0.0, 0.0, 0.0, 0.0);
            }

            RopedBvhNode::NonLeaf {
                bb,
                on_hit_goto_id,
                on_miss_goto_id,
            } => {
                let on_hit_goto_ptr =
                    on_hit_goto_id.map(|id| id * 2).unwrap_or_default();

                let on_miss_goto_ptr =
                    on_miss_goto_id.map(|id| id * 2).unwrap_or_default();

                #[allow(clippy::identity_op)]
                let info = 0
                    | ((on_hit_goto_ptr as u32) << 1)
                    | ((on_miss_goto_ptr as u32) << 16);

                v1 = bb.min().extend(f32::from_bits(info));
                v2 = bb.max().extend(0.0);
            }
        }

        data.push(v1);
        data.push(v2);
    }
}
