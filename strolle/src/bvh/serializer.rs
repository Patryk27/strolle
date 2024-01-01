use glam::Vec4;
use spirv_std::glam::vec4;

use super::{BvhNodeId, BvhNodes};
use crate::instances::Instances;
use crate::primitives::BlasPrimitives;
use crate::{
    gpu, AlphaMode, BvhNode, Materials, Params, Primitive, Primitives,
};

const OP_INTERNAL: u32 = 0;
const OP_LEAF: u32 = 1;

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

    walk_tlas(
        materials,
        instances,
        primitives,
        nodes,
        buffer,
        BvhNodeId::root(),
    );
}

fn walk_tlas<P>(
    materials: &Materials<P>,
    instances: &Instances<P>,
    primitives: &Primitives<P>,
    nodes: &BvhNodes,
    buffer: &mut Vec<Vec4>,
    id: BvhNodeId,
) -> u32
where
    P: Params,
{
    let ptr = buffer.len();

    match nodes[id] {
        BvhNode::Internal {
            left_id, right_id, ..
        } => {
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());

            let _left_ptr = walk_tlas(
                materials, instances, primitives, nodes, buffer, left_id,
            );

            let right_ptr = walk_tlas(
                materials, instances, primitives, nodes, buffer, right_id,
            );

            serialize_internal_node(
                nodes,
                &mut buffer[ptr..],
                left_id,
                right_id,
                right_ptr,
            );
        }

        BvhNode::Leaf { primitives_ref, .. } => {
            let mut processed_one = false;

            for (primitive_idx, primitive) in
                primitives.tlas().index(primitives_ref).iter().enumerate()
            {
                match primitive {
                    Primitive::Triangle {
                        triangle_id,
                        material_id,
                        ..
                    } => {
                        serialize_leaf_node(
                            materials,
                            buffer,
                            *triangle_id,
                            *material_id,
                            primitive_idx,
                            primitives_ref.len(),
                        );
                    }

                    Primitive::Instance { node_id, .. } => {
                        assert!(!processed_one);

                        let primitives =
                            primitives.blas(instances.node_to_handle(*node_id));

                        walk_blas(
                            materials, instances, primitives, nodes, buffer,
                            *node_id,
                        );
                    }

                    Primitive::Killed => {
                        unreachable!();
                    }
                }

                processed_one = true;
            }
        }
    }

    ptr as u32
}

fn walk_blas<P>(
    materials: &Materials<P>,
    instances: &Instances<P>,
    blas: &BlasPrimitives,
    nodes: &BvhNodes,
    buffer: &mut Vec<Vec4>,
    id: BvhNodeId,
) -> u32
where
    P: Params,
{
    let ptr = buffer.len();

    match nodes[id] {
        BvhNode::Internal {
            left_id, right_id, ..
        } => {
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());
            buffer.push(Default::default());

            let _left_ptr =
                walk_blas(materials, instances, blas, nodes, buffer, left_id);

            let right_ptr =
                walk_blas(materials, instances, blas, nodes, buffer, right_id);

            serialize_internal_node(
                nodes,
                &mut buffer[ptr..],
                left_id,
                right_id,
                right_ptr,
            );
        }

        BvhNode::Leaf { primitives_ref, .. } => {
            let triangle_id =
                gpu::TriangleId::new(primitives_ref.start().get());

            let material_id = blas.material_id();

            serialize_leaf_node(
                materials,
                buffer,
                triangle_id,
                material_id,
                0,
                1,
            );
        }
    }

    ptr as u32
}

fn serialize_internal_node(
    nodes: &BvhNodes,
    buffer: &mut [Vec4],
    left_id: BvhNodeId,
    right_id: BvhNodeId,
    right_ptr: u32,
) {
    let left_bb = nodes[left_id].bounds();
    let right_bb = nodes[right_id].bounds();

    buffer[0] = vec4(
        left_bb.min().x,
        left_bb.min().y,
        left_bb.min().z,
        f32::from_bits(OP_INTERNAL),
    );

    buffer[1] = vec4(
        left_bb.max().x,
        left_bb.max().y,
        left_bb.max().z,
        f32::from_bits(right_ptr),
    );

    buffer[2] = vec4(
        right_bb.min().x,
        right_bb.min().y,
        right_bb.min().z,
        Default::default(),
    );

    buffer[3] = vec4(
        right_bb.max().x,
        right_bb.max().y,
        right_bb.max().z,
        Default::default(),
    );
}

fn serialize_leaf_node<P>(
    materials: &Materials<P>,
    buffer: &mut Vec<Vec4>,
    triangle_id: gpu::TriangleId,
    material_id: gpu::MaterialId,
    primitive_idx: usize,
    primitives_len: usize,
) where
    P: Params,
{
    let material = &materials[material_id];

    let flags = {
        let got_more_entries = primitive_idx + 1 < primitives_len;

        let has_alpha_blending =
            matches!(material.alpha_mode, AlphaMode::Blend);

        (got_more_entries as u32) | ((has_alpha_blending as u32) << 1)
    };

    buffer.push(vec4(
        f32::from_bits(flags),
        f32::from_bits(triangle_id.get()),
        f32::from_bits(material_id.get()),
        f32::from_bits(OP_LEAF),
    ));
}
