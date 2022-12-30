mod morton_code;

use spirv_std::glam::Vec3;

use self::morton_code::MortonCode;
use crate::bvh::{BoundingBox, BvhNode, BvhObject};

/// Builds LVBH as described by Kerras in ¹.
///
/// It's a naive CPU implementation supposed to serve as a reference point for
/// our (not yet implemented) GPU one.
///
/// ¹ https://devblogs.nvidia.com/wp-content/uploads/2012/11/karras2012hpg_paper.pdf
pub fn build<T>(objects: &[T]) -> BvhNode
where
    T: BvhObject,
{
    /// Transforms given point into a Morton code.
    ///
    /// Point's coordinates should be within range 0.0..=1.0.
    fn vec3_to_morton(vec: Vec3) -> MortonCode {
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

        MortonCode(xs | ys | zs)
    }

    let scene_bb = BoundingBox::from_objects(objects);
    let mut morton_objects = Vec::new();

    for (object_idx, object) in objects.iter().enumerate() {
        let object_pos = object.center();
        let object_pos = scene_bb.map(object_pos);
        let object_code = vec3_to_morton(object_pos);

        morton_objects.push((object_idx, object_code));
    }

    morton_objects.sort_unstable_by(|(_, morton_a), (_, morton_b)| {
        morton_a.cmp(morton_b)
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
        morton_objects: &[(usize, MortonCode)],
        left: usize,
        right: usize,
    ) -> LinearBvhNode {
        assert!(left <= right);

        if left == right {
            LinearBvhNode::Leaf {
                bb: Default::default(),
                object_idx: morton_objects[left].0,
            }
        } else {
            let split = find_split(morton_objects, left, right);
            let left_node = generate(morton_objects, left, split);
            let right_node = generate(morton_objects, split + 1, right);

            LinearBvhNode::Internal {
                bb: Default::default(),
                left: Box::new(left_node),
                right: Box::new(right_node),
            }
        }
    }

    fn find_split(
        morton_objects: &[(usize, MortonCode)],
        left: usize,
        right: usize,
    ) -> usize {
        let left_code = morton_objects[left].1;
        let right_code = morton_objects[right].1;
        let common_prefix = (left_code ^ right_code).leading_zeros();

        let mut split = left;
        let mut step = right - left;

        loop {
            step = (step + 1) >> 1;

            let middle = split + step;

            if middle < right {
                let middle_code = morton_objects[middle].1;
                let middle_prefix = (left_code ^ middle_code).leading_zeros();

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

    let mut root = generate(&morton_objects, 0, morton_objects.len() - 1);

    root.assign_bounding_boxes(objects);
    root.map(objects)
}

#[derive(Clone, Debug)]
enum LinearBvhNode {
    Internal {
        bb: BoundingBox,
        left: Box<Self>,
        right: Box<Self>,
    },

    Leaf {
        bb: BoundingBox,
        object_idx: usize,
    },
}

impl LinearBvhNode {
    fn assign_bounding_boxes<T>(&mut self, objects: &[T]) -> BoundingBox
    where
        T: BvhObject,
    {
        match self {
            LinearBvhNode::Internal { bb, left, right } => {
                *bb = left.assign_bounding_boxes(objects)
                    + right.assign_bounding_boxes(objects);

                *bb
            }

            LinearBvhNode::Leaf { bb, object_idx } => {
                *bb = objects[*object_idx].bounding_box();
                *bb
            }
        }
    }

    fn map<T>(self, objects: &[T]) -> BvhNode
    where
        T: BvhObject,
    {
        match self {
            LinearBvhNode::Internal { bb, left, right } => BvhNode::Internal {
                bb,
                left: Box::new(left.map(objects)),
                right: Box::new(right.map(objects)),
            },

            LinearBvhNode::Leaf { bb, object_idx } => BvhNode::Leaf {
                bb,
                payload: objects[object_idx].payload(),
            },
        }
    }
}
