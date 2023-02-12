use std::fmt::Write;

use super::BvhNode;

pub struct BvhPrinter;

impl BvhPrinter {
    pub fn print(root: &BvhNode) -> String {
        let mut out = String::new();

        _ = writeln!(&mut out, "digraph {{");
        Self::process(&mut out, &mut 0, root);
        _ = writeln!(&mut out, "}}");

        out
    }

    fn process(
        out: &mut String,
        id_counter: &mut usize,
        node: &BvhNode,
    ) -> usize {
        let id = *id_counter;

        *id_counter += 1;

        match node {
            BvhNode::Internal { bb, left, right } => {
                _ = writeln!(
                    out,
                    "  n{} [label=\"{} .. {}\"]",
                    id,
                    bb.min(),
                    bb.max()
                );

                let left_id = Self::process(out, id_counter, left);
                let right_id = Self::process(out, id_counter, right);

                for child_id in [left_id, right_id] {
                    _ = writeln!(out, "  n{id} -> n{child_id}");
                }
            }

            BvhNode::Leaf {
                bb,
                triangle_id,
                material_id,
            } => {
                _ = writeln!(
                    out,
                    "  n{} [label=\"{} .. {}\\n{:?} / {:?}\"]",
                    id,
                    bb.min(),
                    bb.max(),
                    triangle_id,
                    material_id,
                );
            }
        }

        id
    }
}
