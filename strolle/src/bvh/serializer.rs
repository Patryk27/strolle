use bevy::pbr::AlphaMode;
use glam::Vec4;
use spirv_std::glam::vec4;

use super::{BvhNodeId, BvhNodes, BvhPrimitives};
use crate::{BvhNode, Materials};

pub fn run(
    materials: &Materials,
    nodes: &BvhNodes,
    primitives: &BvhPrimitives,
    buffer: &mut Vec<Vec4>,
) {
    buffer.clear();

    serialize(materials, nodes, primitives, buffer, BvhNodeId::root());
}

fn serialize(
    materials: &Materials,
    nodes: &BvhNodes,
    primitives: &BvhPrimitives,
    buffer: &mut Vec<Vec4>,
    id: BvhNodeId,
) -> u32 {
    const OP_INTERNAL: u32 = 0;
    const OP_LEAF: u32 = 1;

    let ptr = buffer.len();

    match nodes[id] {
        BvhNode::Internal {
            left_id, right_id, ..
        } => {
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());

            let left_bb = nodes[left_id].bounds();
            let right_bb = nodes[right_id].bounds();

            let _left_ptr =
                serialize(materials, nodes, primitives, buffer, left_id);

            let right_ptr =
                serialize(materials, nodes, primitives, buffer, right_id);

            buffer[ptr] = vec4(
                left_bb.min().x,
                left_bb.min().y,
                left_bb.min().z,
                f32::from_bits(OP_INTERNAL),
            );

            buffer[ptr + 1] = vec4(
                left_bb.max().x,
                left_bb.max().y,
                left_bb.max().z,
                f32::from_bits(right_ptr),
            );

            // TODO we could store information about transparency here to
            //      quickly reject nodes during bvh traversal later
            buffer[ptr + 2] = vec4(
                right_bb.min().x,
                right_bb.min().y,
                right_bb.min().z,
                Default::default(),
            );

            buffer[ptr + 3] = vec4(
                right_bb.max().x,
                right_bb.max().y,
                right_bb.max().z,
                Default::default(),
            );
        }

        BvhNode::Leaf { primitives_ref, .. } => {
            for (primitive_idx, primitive) in
                primitives.current(primitives_ref).iter().enumerate()
            {
                let material = &materials[primitive.material_id];

                let flags = {
                    let got_more_entries =
                        primitive_idx + 1 < primitives_ref.len();

                    let has_alpha_blending =
                        matches!(material.alpha_mode, AlphaMode::Blend);

                    (got_more_entries as u32)
                        | ((has_alpha_blending as u32) << 1)
                };

                buffer.push(vec4(
                    f32::from_bits(flags),
                    f32::from_bits(primitive.triangle_id.get()),
                    f32::from_bits(primitive.material_id.get()),
                    f32::from_bits(OP_LEAF),
                ));
            }
        }
    }

    ptr as u32
}
