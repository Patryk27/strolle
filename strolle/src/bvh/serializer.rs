use std::collections::HashMap;

use glam::Vec4;
use spirv_std::glam::vec4;

use super::{BvhNodeId, BvhNodes};
use crate::meshes::Meshes;
use crate::primitives::PrimitiveScope;
use crate::{
    AlphaMode, BvhNode, Materials, Params, Primitive, Primitives,
    ScopedPrimitives,
};

pub fn run<P>(
    meshes: &mut Meshes<P>,
    primitives: &Primitives<P>,
    materials: &Materials<P>,
    nodes: &BvhNodes,
    buffer: &mut Vec<Vec4>,
) where
    P: Params,
{
    buffer.clear();
    buffer.push(Vec4::ZERO);

    let mut links = HashMap::new();

    for (mesh_handle, mesh) in meshes.meshes.iter() {
        if let Some(node_id) = mesh.node_id() {
            let node_ptr = serialize(
                materials,
                nodes,
                primitives.scope(PrimitiveScope::Blas(*mesh_handle)),
                buffer,
                &Default::default(),
                node_id,
            );

            links.insert(node_id.get(), node_ptr);
        }
    }

    let ptr = serialize(
        materials,
        nodes,
        primitives.scope(PrimitiveScope::Tlas),
        buffer,
        &links,
        BvhNodeId::root(),
    );

    buffer[0].x = f32::from_bits(ptr);
}

fn serialize<P>(
    materials: &Materials<P>,
    nodes: &BvhNodes,
    primitives: &ScopedPrimitives<P>,
    buffer: &mut Vec<Vec4>,
    links: &HashMap<u32, u32>,
    id: BvhNodeId,
) -> u32
where
    P: Params,
{
    const OP_INTERNAL: u32 = 0;
    const OP_LEAF_TRIANGLE: u32 = 1;
    const OP_LEAF_INSTANCE: u32 = 2;

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
                serialize(materials, nodes, primitives, buffer, links, left_id);

            let right_ptr = serialize(
                materials, nodes, primitives, buffer, links, right_id,
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
            for (primitive_idx, primitive) in
                primitives.current(primitives_ref).iter().enumerate()
            {
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
                            f32::from_bits(OP_LEAF_TRIANGLE),
                        ));
                    }

                    Primitive::Instance {
                        xform_inv, node_id, ..
                    } => {
                        let flags = {
                            let got_more_entries =
                                primitive_idx + 1 < primitives_ref.len();

                            got_more_entries as u32
                        };

                        buffer.push(vec4(
                            f32::from_bits(flags),
                            f32::from_bits(links[&node_id.get()]),
                            f32::from_bits(0),
                            f32::from_bits(OP_LEAF_INSTANCE),
                        ));

                        buffer.push(
                            xform_inv
                                .matrix3
                                .x_axis
                                .extend(xform_inv.translation.x),
                        );

                        buffer.push(
                            xform_inv
                                .matrix3
                                .y_axis
                                .extend(xform_inv.translation.y),
                        );

                        buffer.push(
                            xform_inv
                                .matrix3
                                .z_axis
                                .extend(xform_inv.translation.z),
                        );
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
