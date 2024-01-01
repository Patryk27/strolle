use std::ops::BitXor;

use glam::Vec3;

use crate::bvh::{BvhNode, BvhNodeHash, BvhNodeId, BvhNodes};
use crate::primitive::{PrimitiveId, PrimitivesRef};
use crate::primitives::BlasPrimitives;
use crate::triangles::Triangles;
use crate::{BoundingBox, Params};

pub fn run<P>(
    triangles: &Triangles<P>,
    blas: &BlasPrimitives,
    nodes: &mut BvhNodes,
) -> BvhNodeId
where
    P: Params,
{
    let mut prims: Vec<_> = blas
        .iter(triangles)
        .map(|(prim_id, prim_bounds, prim_center)| {
            let prim_code = vec3_to_morton(blas.bounds().map(prim_center));

            (prim_id, prim_bounds, prim_code)
        })
        .collect();

    prims.sort_unstable_by(|(_, _, prim_code_a), (_, _, prim_code_b)| {
        prim_code_a.cmp(prim_code_b)
    });

    walk(nodes, &prims).0
}

fn vec3_to_morton(vec: Vec3) -> MortonCode {
    fn expand_bits(mut x: u64) -> u64 {
        x &= 0x1fffff;
        x = (x | x << 32) & 0x1f00000000ffff;
        x = (x | x << 16) & 0x1f0000ff0000ff;
        x = (x | x << 8) & 0x100f00f00f00f00f;
        x = (x | x << 4) & 0x10c30c30c30c30c3;
        x = (x | x << 2) & 0x1249249249249249;
        x
    }

    let resolution = 2.0f32.powi(20);
    let xs = (vec.x * resolution) as u64;
    let ys = (vec.y * resolution) as u64;
    let zs = (vec.z * resolution) as u64;

    let xs = expand_bits(xs);
    let ys = expand_bits(ys) << 2;
    let zs = expand_bits(zs) << 1;

    MortonCode(xs | ys | zs)
}

fn walk(
    nodes: &mut BvhNodes,
    prims: &[(PrimitiveId, BoundingBox, MortonCode)],
) -> (BvhNodeId, BoundingBox) {
    let node;
    let bounds;

    if prims.len() == 1 {
        bounds = prims[0].1;

        node = BvhNode::Leaf {
            bounds,
            primitives_ref: PrimitivesRef::single(prims[0].0),
        };
    } else {
        let split = find_split(prims) + 1;
        let (left_id, left_bb) = walk(nodes, &prims[..split]);
        let (right_id, right_bb) = walk(nodes, &prims[split..]);

        bounds = left_bb + right_bb;

        node = BvhNode::Internal {
            bounds,
            primitives_ref: Default::default(),
            left_id,
            left_hash: BvhNodeHash::new(0),
            right_id,
            right_hash: BvhNodeHash::new(0),
        };
    };

    (nodes.add(node), bounds)
}

fn find_split(prims: &[(PrimitiveId, BoundingBox, MortonCode)]) -> usize {
    let left_code = prims.first().unwrap().2;
    let right_code = prims.last().unwrap().2;
    let common_prefix = (left_code ^ right_code).leading_zeros();

    let mut split = 0;
    let mut step = prims.len();

    loop {
        step = (step + 1) >> 1;

        let middle = split + step;

        if middle < prims.len() {
            let middle_code = prims[middle].2;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MortonCode(u64);

impl BitXor for MortonCode {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl MortonCode {
    fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
}
