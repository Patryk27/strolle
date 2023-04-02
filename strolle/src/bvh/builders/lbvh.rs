mod morton_code;

use spirv_std::glam::Vec3;

use self::morton_code::MortonCode;
use crate::{gpu, BoundingBox, BvhNode, BvhTriangle};

/// Builds LVBH as described by Kerras in ¹.
///
/// ¹ https://devblogs.nvidia.com/wp-content/uploads/2012/11/karras2012hpg_paper.pdf
pub fn build(
    triangles: impl IntoIterator<Item = BvhTriangle> + Clone,
) -> BvhNode {
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

        assert!(vec.x >= 0.0 && vec.x <= 1.0, "Point out of range: {vec:?}");
        assert!(vec.y >= 0.0 && vec.y <= 1.0, "Point out of range: {vec:?}");
        assert!(vec.z >= 0.0 && vec.z <= 1.0, "Point out of range: {vec:?}");

        let resolution = 2.0f32.powi(20);
        let xs = (vec.x * resolution) as u64;
        let ys = (vec.y * resolution) as u64;
        let zs = (vec.z * resolution) as u64;

        let xs = expand_bits(xs);
        let ys = expand_bits(ys) << 2;
        let zs = expand_bits(zs) << 1;

        MortonCode(xs | ys | zs)
    }

    let scene_bb = BoundingBox::from_triangles(
        triangles
            .clone()
            .into_iter()
            .map(|triangle| triangle.triangle),
    );

    let mut triangles: Vec<_> = triangles
        .into_iter()
        .map(|triangle| {
            let BvhTriangle {
                triangle,
                triangle_id,
                material_id,
            } = triangle;

            let morton_code = vec3_to_morton(scene_bb.map(triangle.center()));

            MortonTriangle {
                triangle,
                triangle_id,
                material_id,
                morton_code,
            }
        })
        .collect();

    triangles.sort_unstable_by(|a, b| a.morton_code.cmp(&b.morton_code));

    // TODO
    // for tris in tris.windows(2) {
    //     assert!(
    //         tris[0].0 != tris[1].0,
    //         "Missing feature: support for non-unique Morton codes"
    //     );
    // }

    // -----

    fn generate(
        triangles: &[MortonTriangle],
        left: usize,
        right: usize,
    ) -> LinearBvhNode {
        assert!(left <= right);

        if left == right {
            let triangle = &triangles[left];

            LinearBvhNode::Leaf {
                bb: BoundingBox::from_triangle(triangle.triangle),
                triangle_id: triangle.triangle_id,
                material_id: triangle.material_id,
            }
        } else {
            let split = find_split(triangles, left, right);
            let left_node = generate(triangles, left, split);
            let right_node = generate(triangles, split + 1, right);

            LinearBvhNode::Internal {
                bb: left_node.bb() + right_node.bb(),
                left: Box::new(left_node),
                right: Box::new(right_node),
            }
        }
    }

    fn find_split(
        triangles: &[MortonTriangle],
        left: usize,
        right: usize,
    ) -> usize {
        let left_code = triangles[left].morton_code;
        let right_code = triangles[right].morton_code;
        let common_prefix = (left_code ^ right_code).leading_zeros();

        let mut split = left;
        let mut step = right - left;

        loop {
            step = (step + 1) >> 1;

            let middle = split + step;

            if middle < right {
                let middle_code = triangles[middle].morton_code;
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

    generate(&triangles, 0, triangles.len() - 1).map()
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
        triangle_id: gpu::TriangleId,
        material_id: gpu::MaterialId,
    },
}

impl LinearBvhNode {
    fn bb(&self) -> BoundingBox {
        match self {
            LinearBvhNode::Internal { bb, .. } => *bb,
            LinearBvhNode::Leaf { bb, .. } => *bb,
        }
    }

    fn map(self) -> BvhNode {
        match self {
            LinearBvhNode::Internal { bb, left, right } => BvhNode::Internal {
                bb,
                left: Box::new(left.map()),
                right: Box::new(right.map()),
            },

            LinearBvhNode::Leaf {
                bb,
                triangle_id,
                material_id,
            } => BvhNode::Leaf {
                bb,
                triangle_id,
                material_id,
            },
        }
    }
}

struct MortonTriangle {
    triangle: gpu::Triangle,
    triangle_id: gpu::TriangleId,
    material_id: gpu::MaterialId,
    morton_code: MortonCode,
}
