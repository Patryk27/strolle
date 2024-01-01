use glam::Vec4;
use spirv_std::glam::vec4;

use super::{BvhNodeId, BvhNodes};
use crate::instances::Instances;
use crate::{AlphaMode, BvhNode, Materials, Params, Primitive, Primitives};

pub fn run<P>(
    materials: &Materials<P>,
    instances: &Instances<P>,
    primitives: &Primitives<P>,
    nodes: &BvhNodes,
    buffer: &mut Vec<Vec4>,
) where
    P: Params,
{
    buffer.clear();

    serialize(
        materials,
        instances,
        primitives,
        nodes,
        buffer,
        BvhNodeId::root(),
        None,
    );
}

pub fn serialize<P>(
    materials: &Materials<P>,
    instances: &Instances<P>,
    primitives: &Primitives<P>,
    nodes: &BvhNodes,
    buffer: &mut Vec<Vec4>,
    id: BvhNodeId,
    context: Option<BvhNodeId>,
) -> u32
where
    P: Params,
{
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

            let _left_ptr = serialize(
                materials, instances, primitives, nodes, buffer, left_id,
                context,
            );

            let right_ptr = serialize(
                materials, instances, primitives, nodes, buffer, right_id,
                context,
            );

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
            let prims = if let Some(context) = context {
                primitives
                    .blas(instances.node_to_handle(context))
                    .index(primitives_ref)
            } else {
                primitives.tlas().index(primitives_ref)
            };

            let mut t = false;

            for (primitive_idx, primitive) in prims.iter().enumerate() {
                match primitive {
                    Primitive::Triangle {
                        triangle_id,
                        material_id,
                        ..
                    } => {
                        let material = &materials[*material_id];

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
                            f32::from_bits(triangle_id.get()),
                            f32::from_bits(material_id.get()),
                            f32::from_bits(OP_LEAF),
                        ));
                    }

                    Primitive::Instance { node_id, .. } => {
                        assert!(!t);

                        serialize(
                            materials,
                            instances,
                            primitives,
                            nodes,
                            buffer,
                            *node_id,
                            Some(*node_id),
                        );

                        t = true;
                    }

                    Primitive::Killed => {
                        unreachable!();
                    }
                }
            }
        }
    }

    ptr as u32
}
