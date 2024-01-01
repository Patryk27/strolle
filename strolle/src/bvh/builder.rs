use core::f32;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::thread;

use fxhash::FxHasher;
use glam::UVec3;

use super::{BvhNode, BvhNodeHash, BvhNodeId, BvhNodes};
use crate::primitives::{BlasPrimitives, PrimitivesAccessor, TlasPrimitives};
use crate::{Axis, BoundingBox, Params, PrimitiveId, PrimitivesRef};

const BINS: usize = 12;

pub fn run_blas(
    primitives: &mut BlasPrimitives,
    nodes: &mut BvhNodes,
) -> BvhNodeId {
    let id = nodes.add(BvhNode::Leaf {
        bounds: Default::default(),
        primitives_ref: primitives.all(),
    });

    run::<false>(primitives, nodes, BvhNodeRef { id, ghost: None });

    id
}

pub fn run_tlas<P>(primitives: &mut TlasPrimitives<P>, nodes: &mut BvhNodes)
where
    P: Params,
{
    let prev_root = nodes.update_root(BvhNode::Leaf {
        bounds: Default::default(),
        primitives_ref: primitives.all(),
    });

    run::<true>(
        primitives,
        nodes,
        BvhNodeRef {
            id: BvhNodeId::root(),
            ghost: prev_root,
        },
    );
}

fn run<const TLAS: bool>(
    primitives: &mut impl PrimitivesAccessor,
    nodes: &mut BvhNodes,
    root: BvhNodeRef,
) {
    thread::scope(|s| {
        s.spawn(|| {
            let mut stack = VecDeque::from_iter([root]);

            while let Some(node) = stack.pop_front() {
                match balance::<TLAS>(primitives, nodes, node) {
                    (Some(left), Some(right)) => {
                        stack.push_back(left);
                        stack.push_back(right);
                    }
                    (Some(node), None) | (None, Some(node)) => {
                        stack.push_back(node);
                    }
                    (None, None) => {
                        //
                    }
                }
            }
        });
    });
}

#[inline(always)]
fn balance<const TLAS: bool>(
    primitives: &mut impl PrimitivesAccessor,
    nodes: &mut BvhNodes,
    node_ref: BvhNodeRef,
) -> (Option<BvhNodeRef>, Option<BvhNodeRef>) {
    if let Some(plane) = find_splitting_plane(primitives, nodes, node_ref.id) {
        if plane.split_cost < nodes[node_ref.id].sah_cost() {
            return split::<TLAS>(nodes, primitives, node_ref, plane);
        }
    }

    if TLAS {
        if let Some(BvhNode::Internal {
            left_id, right_id, ..
        }) = node_ref.ghost
        {
            nodes.remove_tree(left_id);
            nodes.remove_tree(right_id);
        }
    }

    (None, None)
}

#[inline(always)]
fn find_splitting_plane(
    primitives: &impl PrimitivesAccessor,
    nodes: &BvhNodes,
    node_id: BvhNodeId,
) -> Option<SplittingPlane> {
    let BvhNode::Leaf { primitives_ref, .. } = nodes[node_id] else {
        unreachable!();
    };

    if primitives_ref.len() <= 1 {
        return None;
    }

    let primitives = primitives.index(primitives_ref);

    // ---

    let centroid_bb: BoundingBox = primitives
        .iter()
        .map(|primitive| primitive.center())
        .collect();

    let mut bins = [[Bin::default(); BINS]; 3];
    let scale = (BINS as f32) / centroid_bb.extent();

    for primitive in primitives {
        let bin_id = scale * (primitive.center() - centroid_bb.min());
        let bin_id = bin_id.as_uvec3().min(UVec3::splat((BINS as u32) - 1));
        let bin_idx = bin_id.x as usize;
        let bin_idy = bin_id.y as usize;
        let bin_idz = bin_id.z as usize;

        bins[0][bin_idx].count += 1;
        bins[0][bin_idx].bounds += primitive.bounds();

        bins[1][bin_idy].count += 1;
        bins[1][bin_idy].bounds += primitive.bounds();

        bins[2][bin_idz].count += 1;
        bins[2][bin_idz].bounds += primitive.bounds();
    }

    // ---

    let mut left_areas = [[0.0; BINS - 1]; 3];
    let mut right_areas = [[0.0; BINS - 1]; 3];
    let mut left_counts = [[0; BINS - 1]; 3];
    let mut right_counts = [[0; BINS - 1]; 3];
    let mut left_bb = [BoundingBox::default(); 3];
    let mut right_bb = [BoundingBox::default(); 3];
    let mut left_count = [0; 3];
    let mut right_count = [0; 3];

    for axis in 0..3 {
        for i in 0..(BINS - 1) {
            let left_bin = bins[axis][i];

            left_count[axis] += left_bin.count;
            left_counts[axis][i] = left_count[axis];

            if left_bin.bounds.is_set() {
                left_bb[axis] += left_bin.bounds;
            }

            left_areas[axis][i] = left_bb[axis].half_area();

            // ---

            let right_bin = bins[axis][BINS - 1 - i];

            right_count[axis] += right_bin.count;
            right_counts[axis][BINS - 2 - i] = right_count[axis];

            if right_bin.bounds.is_set() {
                right_bb[axis] += right_bin.bounds;
            }

            right_areas[axis][BINS - 2 - i] = right_bb[axis].half_area();
        }
    }

    // ---

    let mut best: Option<SplittingPlane> = None;
    let scale = centroid_bb.extent() / (BINS as f32);

    for axis in 0..3 {
        for i in 0..(BINS - 1) {
            let split_cost = (left_counts[axis][i] as f32)
                * left_areas[axis][i]
                + (right_counts[axis][i] as f32) * right_areas[axis][i];

            let is_current_bin_better =
                best.map_or(true, |best| split_cost <= best.split_cost);

            if is_current_bin_better {
                let split_by = Axis::from(axis);

                let split_at = centroid_bb.min()[split_by]
                    + scale[split_by] * ((i + 1) as f32);

                best = Some(SplittingPlane {
                    split_by,
                    split_at,
                    split_cost,
                });
            }
        }
    }

    best
}

fn split<const TLAS: bool>(
    nodes: &mut BvhNodes,
    primitives: &mut impl PrimitivesAccessor,
    node_ref: BvhNodeRef,
    plane: SplittingPlane,
) -> (Option<BvhNodeRef>, Option<BvhNodeRef>) {
    let BvhNode::Leaf {
        bounds,
        primitives_ref,
    } = nodes[node_ref.id]
    else {
        unreachable!();
    };

    // ---

    let primitives_data = primitives.index_mut(primitives_ref);

    let mut left_prim_idx = 0;
    let mut right_prim_idx = (primitives_data.len() - 1) as i32;

    // TODO optimization idea: don't compute hashes when close to leaves
    let mut left_hash = FxHasher::default();
    let mut right_hash = FxHasher::default();

    let mut left_bounds = BoundingBox::default();
    let mut right_bounds = BoundingBox::default();

    while left_prim_idx <= right_prim_idx {
        let primitive = primitives_data[left_prim_idx as usize];

        if primitive.center()[plane.split_by] < plane.split_at {
            left_prim_idx += 1;
            left_bounds += primitive.bounds();

            if TLAS {
                primitive.hash(&mut left_hash);
            }
        } else {
            primitives_data
                .swap(left_prim_idx as usize, right_prim_idx as usize);

            right_prim_idx -= 1;
            right_bounds += primitive.bounds();

            if TLAS {
                primitive.hash(&mut right_hash);
            }
        }
    }

    let pivot =
        PrimitiveId::new(primitives_ref.start().get() + (left_prim_idx as u32));

    let left_primitives_ref = PrimitivesRef::new(primitives_ref.start(), pivot);
    let right_primitives_ref = PrimitivesRef::new(pivot, primitives_ref.end());

    let left_hash = BvhNodeHash::new(left_hash.finish());
    let right_hash = BvhNodeHash::new(right_hash.finish());

    // ---

    let mut left_id = None;
    let mut right_id = None;

    let mut left_ghost = None;
    let mut right_ghost = None;

    let mut left_continue = true;
    let mut right_continue = true;

    if TLAS {
        if let Some(BvhNode::Internal {
            left_id: prev_left_id,
            left_hash: prev_left_hash,
            right_id: prev_right_id,
            right_hash: prev_right_hash,
            ..
        }) = node_ref.ghost
        {
            // if prev_left_hash == left_hash {
            //     left_id = Some(prev_left_id);
            //     left_continue = false;

            //     copy(nodes, primitives, prev_left_id, left_primitives_ref);
            // } else {
            left_ghost = Some(nodes.remove(prev_left_id));
            // }

            // if prev_right_hash == right_hash {
            //     right_id = Some(prev_right_id);
            //     right_continue = false;

            //     copy(nodes, primitives, prev_right_id, right_primitives_ref);
            // } else {
            right_ghost = Some(nodes.remove(prev_right_id));
            // }
        }
    }

    // ---

    let left_id = left_id.unwrap_or_else(|| {
        nodes.add(BvhNode::Leaf {
            bounds: left_bounds,
            primitives_ref: left_primitives_ref,
        })
    });

    let right_id = right_id.unwrap_or_else(|| {
        nodes.add(BvhNode::Leaf {
            bounds: right_bounds,
            primitives_ref: right_primitives_ref,
        })
    });

    nodes[node_ref.id] = BvhNode::Internal {
        bounds,
        primitives_ref,
        left_id,
        left_hash,
        right_id,
        right_hash,
    };

    // ---

    let left = left_continue.then_some(BvhNodeRef {
        id: left_id,
        ghost: left_ghost,
    });

    let right = right_continue.then_some(BvhNodeRef {
        id: right_id,
        ghost: right_ghost,
    });

    (left, right)
}

// fn copy<P>(
//     nodes: &mut BvhNodes,
//     primitives: &mut ScopedPrimitives<P>,
//     id: BvhNodeId,
//     primitives_ref: PrimitivesRef,
// ) where
//     P: Params,
// {
//     let prev_primitives_ref = nodes[id].primitives_ref();

//     primitives.copy_previous_to_current(prev_primitives_ref, primitives_ref);

//     let primitives_offset = (primitives_ref.start().get() as i32)
//         - (prev_primitives_ref.start().get() as i32);

//     if primitives_offset != 0 {
//         offset_primitives(nodes, primitives_offset, id);
//     }
// }

// fn offset_primitives(nodes: &mut BvhNodes, offset: i32, id: BvhNodeId) {
//     match &mut nodes[id] {
//         BvhNode::Internal { primitives_ref, .. }
//         | BvhNode::Leaf { primitives_ref, .. } => {
//             primitives_ref.offset(offset);
//         }
//     }

//     if let BvhNode::Internal {
//         left_id, right_id, ..
//     } = nodes[id]
//     {
//         offset_primitives(nodes, offset, left_id);
//         offset_primitives(nodes, offset, right_id);
//     }
// }

#[derive(Clone, Copy, Debug)]
struct SplittingPlane {
    split_by: Axis,
    split_at: f32,
    split_cost: f32,
}

#[derive(Clone, Copy, Default, Debug)]
struct Bin {
    bounds: BoundingBox,
    count: u32,
}

#[derive(Debug)]
struct BvhNodeRef {
    id: BvhNodeId,
    ghost: Option<BvhNode>,
}
