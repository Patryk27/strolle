use core::f32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;

use crossbeam::channel::{self, Receiver, Sender};

use crate::{Axis, BoundingBox, BvhNode, BvhPrimitive};

/// Number of worker-threads to spawn for processing the tree.
const THREADS: usize = 3;

/// Number of bins to use when looking for the optimal splitting plane¹.
///
/// With a pinch of salt: more is better, with the trade-off on performance and
/// stack size.
///
/// ¹ see: binned SAH
const BINS: usize = 32;

/// Constructs BVH using a multi-threaded binned SAH algorithm.
///
/// Thanks to:
/// https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/.
pub fn run<'a>(
    nodes: Vec<BvhNode<'a>>,
    primitives: &'a mut [BvhPrimitive],
) -> Vec<BvhNode<'a>> {
    let nodes = Mutex::new(nodes);
    let (queue_tx, queue_rx) = channel::unbounded();

    _ = queue_tx.send({
        let bounds = primitives
            .iter()
            .map(|primitive| primitive.bounds)
            .collect();

        WorkerMsg::BalanceNode {
            node_id: 0,
            node: BvhNode::Leaf { bounds, primitives },
        }
    });

    // Number of nodes allocated so far; used to index into `nodes`.
    //
    // We're starting with one because the root node is considered already
    // allocated above.
    let allocated_nodes = AtomicU32::new(1);

    // Number of messages present on the queue; used to know when to shut down
    // the workers.
    //
    // Note that it's not the same as `queue_rx.len()` because the workers
    // decrease this atomic only *after* a message has been processed, as
    // compared to `queue_rx.len()` which gets decreased right after a message
    // gets popped from the queue.
    let pending_messages = AtomicU32::new(1);

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                worker_main(
                    &nodes,
                    &queue_tx,
                    &queue_rx,
                    &allocated_nodes,
                    &pending_messages,
                );
            });
        }
    });

    nodes.into_inner().unwrap()
}

enum WorkerMsg<'a> {
    BalanceNode { node_id: u32, node: BvhNode<'a> },
    Halt,
}

fn worker_main<'a>(
    nodes: &Mutex<Vec<BvhNode<'a>>>,
    queue_tx: &Sender<WorkerMsg<'a>>,
    queue_rx: &Receiver<WorkerMsg<'a>>,
    allocated_nodes: &AtomicU32,
    pending_messages: &AtomicU32,
) {
    while let Ok(WorkerMsg::BalanceNode { node_id, node }) = queue_rx.recv() {
        let (parent, children) = balance(&allocated_nodes, node);

        if let Some((left_id, left, right_id, right)) = children {
            pending_messages.fetch_add(2, Ordering::Relaxed);

            _ = queue_tx.send(WorkerMsg::BalanceNode {
                node_id: left_id,
                node: left,
            });

            _ = queue_tx.send(WorkerMsg::BalanceNode {
                node_id: right_id,
                node: right,
            });
        }

        if let Ok(mut nodes) = nodes.lock() {
            nodes[node_id as usize] = parent;
        }

        if pending_messages.fetch_sub(1, Ordering::Relaxed) == 1 {
            for _ in 0..THREADS {
                _ = queue_tx.send(WorkerMsg::Halt);
            }
        }
    }
}

fn balance<'a>(
    allocated_nodes: &AtomicU32,
    node: BvhNode<'a>,
) -> (BvhNode<'a>, Option<(u32, BvhNode<'a>, u32, BvhNode<'a>)>) {
    if let Some(plane) = find_splitting_plane(&node) {
        if plane.split_cost < node.sah_cost() {
            split(allocated_nodes, node, plane.split_by, plane.split_at)
        } else {
            (node, None)
        }
    } else {
        (node, None)
    }
}

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

fn find_splitting_plane(node: &BvhNode) -> Option<SplittingPlane> {
    let BvhNode::Leaf { primitives, .. } = node else {
        return None;
    };

    if primitives.len() <= 1 {
        return None;
    }

    let mut best: Option<SplittingPlane> = None;

    let centroid_bb: BoundingBox = primitives
        .iter()
        .map(|primitive| primitive.center)
        .collect();

    for split_by in Axis::all() {
        let mut bins = [Bin::default(); BINS];
        let scale = (BINS as f32) / centroid_bb.extent()[split_by];

        for primitive in primitives.iter() {
            let bin_idx = scale
                * (primitive.center[split_by] - centroid_bb.min()[split_by]);

            let bin_idx = (bin_idx as usize).min(BINS - 1);

            bins[bin_idx].bounds += primitive.bounds;
            bins[bin_idx].count += 1;
        }

        // ---

        let mut left_areas = [0.0; BINS - 1];
        let mut right_areas = [0.0; BINS - 1];
        let mut left_counts = [0; BINS - 1];
        let mut right_counts = [0; BINS - 1];
        let mut left_bb = BoundingBox::default();
        let mut right_bb = BoundingBox::default();
        let mut left_count = 0;
        let mut right_count = 0;

        for i in 0..(BINS - 1) {
            left_count += bins[i].count;
            left_counts[i] = left_count;

            left_bb += bins[i].bounds;
            left_areas[i] = left_bb.half_area();

            right_count += bins[BINS - 1 - i].count;
            right_counts[BINS - 2 - i] = right_count;

            right_bb += bins[BINS - 1 - i].bounds;
            right_areas[BINS - 2 - i] = right_bb.half_area();
        }

        // ---

        let scale = centroid_bb.extent()[split_by] / (BINS as f32);

        for i in 0..(BINS - 1) {
            let split_cost = (left_counts[i] as f32) * left_areas[i]
                + (right_counts[i] as f32) * right_areas[i];

            let is_current_bin_better =
                best.map_or(true, |best| split_cost <= best.split_cost);

            if is_current_bin_better {
                let split_at =
                    centroid_bb.min()[split_by] + scale * ((i + 1) as f32);

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

fn split<'a>(
    allocated_nodes: &AtomicU32,
    node: BvhNode<'a>,
    split_by: Axis,
    split_at: f32,
) -> (BvhNode<'a>, Option<(u32, BvhNode<'a>, u32, BvhNode<'a>)>) {
    let BvhNode::Leaf { bounds, primitives } = node else {
        return (node, None);
    };

    // ---

    let mut i = 0 as i32;
    let mut j = (primitives.len() - 1) as i32;

    let mut left_bounds = BoundingBox::default();
    let mut right_bounds = BoundingBox::default();

    while i <= j {
        let primitive = primitives[i as usize];

        if primitive.center[split_by] < split_at {
            i += 1;
            left_bounds += primitive.bounds;
        } else {
            primitives.swap(i as usize, j as usize);
            j -= 1;
            right_bounds += primitive.bounds;
        }
    }

    // ---

    let (left_prims, right_prims) = primitives.split_at_mut(i as usize);
    let left_id = allocated_nodes.fetch_add(2, Ordering::Relaxed);
    let right_id = left_id + 1;

    let parent = BvhNode::Internal {
        bounds,
        left_node_id: left_id,
    };

    let left = BvhNode::Leaf {
        bounds: left_bounds,
        primitives: left_prims,
    };

    let right = BvhNode::Leaf {
        bounds: right_bounds,
        primitives: right_prims,
    };

    (parent, Some((left_id, left, right_id, right)))
}
