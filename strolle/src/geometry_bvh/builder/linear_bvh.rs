//! This module implements LBVH as described by Kerras in ¹.
//!
//! It's a naive CPU implementation supposed to serve as a reference point for
//! our (not yet implemented) GPU one.
//!
//! ¹ https://devblogs.nvidia.com/wp-content/uploads/2012/11/karras2012hpg_paper.pdf

use spirv_std::glam::Vec3;
use strolle_models::TriangleId;

use super::*;
use crate::GeometryTris;

#[derive(Clone)]
pub struct LinearBvh;

impl LinearBvh {
    /// See the module documentation for reference.
    pub fn build(scene: &GeometryTris) -> BvhNode {
        /// Transforms given point into a Morton code.
        ///
        /// Point's coordinates should be within range 0.0..=1.0.
        fn vec3_to_morton(vec: Vec3) -> u64 {
            /// Expands a 21-bit number into a 64-bit one by inserting zeros
            /// between bits.
            fn expand_bits(mut x: u64) -> u64 {
                x &= 0x1fffff;
                x = (x | x << 32) & 0x1f00000000ffff;
                x = (x | x << 16) & 0x1f0000ff0000ff;
                x = (x | x << 8) & 0x100f00f00f00f00f;
                x = (x | x << 4) & 0x10c30c30c30c30c3;
                x = (x | x << 2) & 0x1249249249249249;
                x
            }

            assert!(
                vec.x >= 0.0 && vec.x <= 1.0,
                "Point out of range: {:?}",
                vec
            );
            assert!(
                vec.y >= 0.0 && vec.y <= 1.0,
                "Point out of range: {:?}",
                vec
            );
            assert!(
                vec.z >= 0.0 && vec.z <= 1.0,
                "Point out of range: {:?}",
                vec
            );

            let resolution = 2.0f32.powi(20);
            let xs = (vec.x * resolution) as u64;
            let ys = (vec.y * resolution) as u64;
            let zs = (vec.z * resolution) as u64;

            let xs = expand_bits(xs);
            let ys = expand_bits(ys) << 2;
            let zs = expand_bits(zs) << 1;

            xs | ys | zs
        }

        let scene_bb = BoundingBox::for_scene(scene);
        let mut tris = Vec::new();

        for (tri_id, tri) in scene.iter() {
            let tri_code = vec3_to_morton(scene_bb.map(tri.center()));

            tris.push((tri_code, tri_id));
        }

        tris.sort_unstable_by(|(tri_code_a, _), (tri_code_b, _)| {
            tri_code_a.cmp(tri_code_b)
        });

        // TODO
        // for tris in tris.windows(2) {
        //     assert!(
        //         tris[0].0 != tris[1].0,
        //         "Missing feature: support for non-unique Morton codes"
        //     );
        // }

        // -----

        fn generate(
            tris: &[(u64, TriangleId)],
            left: usize,
            right: usize,
        ) -> LinearBvhNode {
            assert!(!tris.is_empty());
            assert!(left <= right);

            if left == right {
                LinearBvhNode::Leaf {
                    bb: Default::default(),
                    tri: tris[left].1,
                }
            } else {
                let split = find_split(tris, left, right);
                let left_node = generate(tris, left, split);
                let right_node = generate(tris, split + 1, right);

                LinearBvhNode::Node {
                    bb: Default::default(),
                    left: Box::new(left_node),
                    right: Box::new(right_node),
                }
            }
        }

        fn find_split(
            tris: &[(u64, TriangleId)],
            left: usize,
            right: usize,
        ) -> usize {
            let left_code = tris[left].0;
            let right_code = tris[right].0;
            let common_prefix = (left_code ^ right_code).leading_zeros();

            let mut split = left;
            let mut step = right - left;

            loop {
                step = (step + 1) >> 1;

                let middle = split + step;

                if middle < right {
                    let middle_code = tris[middle].0;

                    let middle_prefix =
                        (left_code ^ middle_code).leading_zeros();

                    if middle_prefix > common_prefix {
                        split = middle;
                    }
                }

                if step <= 1 {
                    break;
                }
            }

            split
        }

        let mut root = generate(&tris, 0, tris.len() - 1);

        root.assign_bounding_boxes(scene);
        root.map()
    }
}

#[derive(Clone, Debug)]
enum LinearBvhNode {
    Node {
        bb: BoundingBox,
        left: Box<Self>,
        right: Box<Self>,
    },

    Leaf {
        bb: BoundingBox,
        tri: TriangleId,
    },
}

impl LinearBvhNode {
    fn assign_bounding_boxes(&mut self, scene: &GeometryTris) -> BoundingBox {
        match self {
            LinearBvhNode::Node { bb, left, right } => {
                *bb = left.assign_bounding_boxes(scene)
                    + right.assign_bounding_boxes(scene);

                *bb
            }

            LinearBvhNode::Leaf { bb, tri } => {
                *bb = scene
                    .get(*tri)
                    .vertices()
                    .into_iter()
                    .fold(BoundingBox::default(), BoundingBox::with);

                *bb
            }
        }
    }

    fn map(self) -> BvhNode {
        match self {
            LinearBvhNode::Node { bb, left, right } => BvhNode::Node {
                bb,
                left: Box::new(left.map()),
                right: Box::new(right.map()),
            },

            LinearBvhNode::Leaf { bb, tri } => BvhNode::Leaf { bb, tri },
        }
    }
}
