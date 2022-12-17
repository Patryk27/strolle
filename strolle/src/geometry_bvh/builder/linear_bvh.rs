//! This module implements LBVH as described by Kerras in ¹.
//!
//! It's a naive CPU implementation supposed to serve as a reference point for
//! our (not yet implemented) GPU one.
//!
//! ¹ https://devblogs.nvidia.com/wp-content/uploads/2012/11/karras2012hpg_paper.pdf

use spirv_std::glam::Vec3;
use strolle_raytracer_models::TriangleId;

use super::*;
use crate::GeometryTris;

#[derive(Clone)]
pub struct LinearBvh;

impl LinearBvh {
    /// See the module documentation for reference.
    pub fn build(scene: &GeometryTris) -> BvhNode {
        fn vec3_to_morton(vec: Vec3) -> u32 {
            fn expand_bits(mut v: u32) -> u32 {
                v = (v * 0x00010001) & 0xFF0000FF;
                v = (v * 0x00000101) & 0x0F00F00F;
                v = (v * 0x00000011) & 0xC30C30C3;
                v = (v * 0x00000005) & 0x49249249;
                v
            }

            // TODO use scene's AABB
            let vec = vec / 20.0 + Vec3::splat(0.5);

            assert!(vec.x >= 0.0 && vec.x <= 1.0);
            assert!(vec.y >= 0.0 && vec.y <= 1.0);
            assert!(vec.z >= 0.0 && vec.z <= 1.0);

            let vec = (vec * 1024.0).as_uvec3();
            let xs = expand_bits(vec.x);
            let ys = expand_bits(vec.y);
            let zs = expand_bits(vec.z);

            xs * 4 + ys * 2 + zs
        }

        let mut tris = Vec::new();

        for (tri_id, tri) in scene.iter() {
            tris.push((vec3_to_morton(tri.center()), tri_id));
        }

        tris.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        // TODO
        // for mt in tris.windows(2) {
        // assert!(mt[0].0 != mt[1].0);
        // }

        // -----

        fn generate(
            tris: &[(u32, TriangleId)],
            left: usize,
            right: usize,
        ) -> LinearBvhNode {
            assert!(!tris.is_empty());

            if left == right {
                LinearBvhNode::Leaf {
                    tris: vec![tris[left].1],
                }
            } else {
                let split = find_split(tris, left, right);
                let left = generate(tris, left, split);
                let right = generate(tris, split + 1, right);

                LinearBvhNode::Node {
                    bb: Default::default(),
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
        }

        fn find_split(
            tris: &[(u32, TriangleId)],
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

                let new_split = split + step;

                if new_split < right {
                    let split_code = tris[new_split].0;
                    let split_prefix = (left_code ^ split_code).leading_zeros();

                    if split_prefix > common_prefix {
                        split = new_split;
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
        root.optimize(None);
        root.map()
    }
}

#[derive(Clone, Debug)]
enum LinearBvhNode {
    Leaf {
        tris: Vec<TriangleId>,
    },

    Node {
        bb: BoundingBox,
        left: Box<Self>,
        right: Box<Self>,
    },
}

impl LinearBvhNode {
    fn assign_bounding_boxes(&mut self, scene: &GeometryTris) -> BoundingBox {
        match self {
            LinearBvhNode::Leaf { tris } => tris
                .iter()
                .flat_map(|tri_id| scene.get(*tri_id).vertices())
                .fold(BoundingBox::default(), BoundingBox::with),

            LinearBvhNode::Node { bb, left, right } => {
                *bb = left.assign_bounding_boxes(scene)
                    + right.assign_bounding_boxes(scene);

                *bb
            }
        }
    }

    fn optimize(&mut self, parent_bb: Option<BoundingBox>) {
        let LinearBvhNode::Node {
            bb,
            left,
            right,
        } = self else { return };

        left.optimize(Some(*bb));
        right.optimize(Some(*bb));

        let Some(parent_bb) = parent_bb else { return };

        if parent_bb != *bb {
            return;
        }

        match (&mut **left, &mut **right) {
            (
                LinearBvhNode::Leaf { tris: left_tris },
                LinearBvhNode::Leaf { tris: right_tris },
            ) => {
                *self = LinearBvhNode::Leaf {
                    tris: left_tris
                        .drain(..)
                        .chain(right_tris.drain(..))
                        .collect(),
                };
            }

            _ => {
                //
            }
        }
    }

    fn map(self) -> BvhNode {
        match self {
            LinearBvhNode::Leaf { tris } => BvhNode::Leaf { tris },

            LinearBvhNode::Node { bb, left, right } => BvhNode::Node {
                bb,
                left: Box::new(left.map()),
                right: Box::new(right.map()),
            },
        }
    }
}
