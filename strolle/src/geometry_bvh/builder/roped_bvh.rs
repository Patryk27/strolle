//! This module implements roped BVH which allows for stackless tree traversal
//! later in the shader.

use std::fmt;

use spirv_std::glam::{vec4, Vec4};
use strolle_raytracer_models::TriangleId;

use super::*;

#[derive(Default)]
pub struct RopedBvh {
    nodes: Vec<RopedBvhNode>,
}

impl RopedBvh {
    pub fn build(root: &BvhNode) -> Self {
        let mut this = Self::default();

        this.add(root, None);
        this
    }

    fn add(&mut self, node: &BvhNode, backtrack_to: Option<usize>) -> usize {
        match node {
            BvhNode::Leaf { tris } => {
                let mut id = None;
                let mut prev_node_id = None;

                for tri in tris {
                    let node_id = self.add_leaf(*tri);

                    if id.is_none() {
                        id = Some(node_id);
                    }

                    if let Some(prev_node_id) = prev_node_id {
                        self.fixup_leaf(prev_node_id, node_id);
                    }

                    prev_node_id = Some(node_id);
                }

                if let Some(backtrack_to) = backtrack_to {
                    self.fixup_leaf(prev_node_id.unwrap(), backtrack_to);
                }

                id.unwrap()
            }

            BvhNode::Node { bb, left, right } => {
                let id = self.add_node(*bb);
                let right_id = self.add(right, backtrack_to);
                let left_id = self.add(left, Some(right_id));

                self.fixup_node(id, left_id, backtrack_to);

                id
            }
        }
    }

    fn add_leaf(&mut self, triangle: TriangleId) -> usize {
        let id = self.nodes.len();

        self.nodes.push(RopedBvhNode::Leaf {
            triangle,
            goto_id: None,
        });

        id
    }

    fn fixup_leaf(&mut self, id: usize, goto_id_val: usize) {
        match &mut self.nodes[id] {
            RopedBvhNode::Leaf { goto_id, .. } => {
                *goto_id = Some(goto_id_val);
            }
            _ => unreachable!(),
        }
    }

    fn add_node(&mut self, bb: BoundingBox) -> usize {
        let id = self.nodes.len();

        self.nodes.push(RopedBvhNode::Node {
            bb,
            on_hit_goto_id: None,
            on_miss_goto_id: None,
        });

        id
    }

    fn fixup_node(
        &mut self,
        id: usize,
        on_hit_goto_id_val: usize,
        on_miss_goto_it_val: Option<usize>,
    ) {
        match &mut self.nodes[id] {
            RopedBvhNode::Node {
                on_hit_goto_id,
                on_miss_goto_id,
                ..
            } => {
                *on_hit_goto_id = Some(on_hit_goto_id_val);
                *on_miss_goto_id = on_miss_goto_it_val;
            }
            _ => unreachable!(),
        }
    }

    pub fn serialize_into(&self, out: &mut Vec<Vec4>) {
        for node in &self.nodes {
            let v1;
            let v2;

            match node {
                RopedBvhNode::Leaf { triangle, goto_id } => {
                    let goto_ptr = goto_id.map(|id| id * 2).unwrap_or_default();

                    let i1 = 1 | ((triangle.get() as u32) << 1);
                    let i2 = goto_ptr as u32;

                    v1 = vec4(0.0, 0.0, 0.0, f32::from_bits(i1));
                    v2 = vec4(0.0, 0.0, 0.0, f32::from_bits(i2));
                }

                RopedBvhNode::Node {
                    bb,
                    on_hit_goto_id,
                    on_miss_goto_id,
                } => {
                    let on_hit_goto_ptr =
                        on_hit_goto_id.map(|id| id * 2).unwrap_or_default();

                    let on_miss_goto_ptr =
                        on_miss_goto_id.map(|id| id * 2).unwrap_or_default();

                    let i1 = (on_hit_goto_ptr as u32) << 1;
                    let i2 = on_miss_goto_ptr as u32;

                    v1 = bb.min().extend(f32::from_bits(i1));
                    v2 = bb.max().extend(f32::from_bits(i2));
                }
            }

            out.push(v1);
            out.push(v2);
        }
    }
}

impl fmt::Display for RopedBvh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "digraph G {{")?;

        for (node_id, node) in self.nodes.iter().enumerate() {
            writeln!(f, "{}", node.print(node_id))?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}

pub enum RopedBvhNode {
    Leaf {
        triangle: TriangleId,
        goto_id: Option<usize>,
    },

    Node {
        bb: BoundingBox,
        on_hit_goto_id: Option<usize>,
        on_miss_goto_id: Option<usize>,
    },
}

impl RopedBvhNode {
    fn print(&self, id: usize) -> String {
        use std::fmt::Write;

        let mut out = String::new();

        match self {
            RopedBvhNode::Leaf { triangle, goto_id } => {
                _ = writeln!(
                    &mut out,
                    "  n{} [label=\"leaf({})\"]",
                    id, triangle
                );

                if let Some(id2) = goto_id {
                    _ = writeln!(
                        &mut out,
                        "  n{} -> n{} [label=\"goto\"]",
                        id, id2
                    );
                }
            }

            RopedBvhNode::Node {
                bb,
                on_hit_goto_id,
                on_miss_goto_id,
            } => {
                _ = writeln!(
                    &mut out,
                    "  n{} [label=\"node({} .. {})\"]",
                    id,
                    bb.min(),
                    bb.max()
                );

                if let Some(id2) = on_hit_goto_id {
                    _ = writeln!(
                        &mut out,
                        "  n{} -> n{} [label=\"on-hit\"]",
                        id, id2
                    );
                }

                if let Some(id2) = on_miss_goto_id {
                    _ = writeln!(
                        &mut out,
                        "  n{} -> n{} [label=\"on-miss\"]",
                        id, id2
                    );
                }
            }
        }

        out
    }
}
