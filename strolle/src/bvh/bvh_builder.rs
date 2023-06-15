use core::f32;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

use ordered_float::OrderedFloat;

use crate::{Axis, BoundingBox, BvhNode, BvhTriangle};

/// Builds BVH using SAH.
///
/// Special thanks to:
/// - https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/.
pub fn build(
    previous: Option<&BvhNode>,
    triangles: impl IntoIterator<Item = BvhTriangle>,
) -> BvhNode {
    let mut root = SahBvhNode::default();

    for triangle in triangles {
        root.add(triangle);
    }

    root.balance(previous);
    root.map()
}

#[derive(Default)]
struct SahBvhNode {
    bb: BoundingBox,
    tris: Vec<BvhTriangle>,
    children: Option<[(Box<Self>, u64); 2]>,
    previous: Option<BvhNode>,
}

impl SahBvhNode {
    fn add(&mut self, triangle: BvhTriangle) {
        self.bb = self.bb + triangle.bb;
        self.tris.push(triangle);
    }

    fn balance(&mut self, prev: Option<&BvhNode>) {
        if let Some((split_by, split_at, split_cost)) =
            self.find_splitting_plane()
        {
            let current_cost = (self.tris.len() as f32) * self.bb.area();

            if split_cost < current_cost {
                self.split(prev, split_by, split_at);
            }
        }
    }

    fn find_splitting_plane(&self) -> Option<(Axis, f32, f32)> {
        const BINS: usize = 32;

        #[derive(Clone, Copy, Default, Debug)]
        struct SahBin {
            bb: BoundingBox,
            count: usize,
        }

        if self.tris.len() <= 1 {
            return None;
        }

        let mut best: Option<(Axis, f32, f32)> = None;
        let mut centroid_bb = BoundingBox::default();

        for triangle in &self.tris {
            centroid_bb.grow(triangle.center);
        }

        for split_by in Axis::all() {
            let mut bins = [SahBin::default(); BINS];
            let scale = (BINS as f32) / centroid_bb.extent()[split_by];

            for triangle in &self.tris {
                let bin_idx = (triangle.center[split_by]
                    - centroid_bb.min()[split_by])
                    * scale;

                let bin_idx = (bin_idx as usize).min(BINS - 1);

                bins[bin_idx].bb = bins[bin_idx].bb + triangle.bb;
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

                left_bb = left_bb + bins[i].bb;
                left_areas[i] = left_bb.area();

                right_count += bins[BINS - 1 - i].count;
                right_counts[BINS - 2 - i] = right_count;

                right_bb = right_bb + bins[BINS - 1 - i].bb;
                right_areas[BINS - 2 - i] = right_bb.area();
            }

            // ---

            let scale = centroid_bb.extent()[split_by] / (BINS as f32);

            for i in 0..(BINS - 1) {
                let split_cost = (left_counts[i] as f32) * left_areas[i]
                    + (right_counts[i] as f32) * right_areas[i];

                if split_cost == 0.0 {
                    continue;
                }

                let is_current_bin_better = best
                    .map_or(true, |(_, _, best_cost)| split_cost < best_cost);

                if is_current_bin_better {
                    let split_at =
                        centroid_bb.min()[split_by] + scale * ((i + 1) as f32);

                    best = Some((split_by, split_at, split_cost));
                }
            }
        }

        best
    }

    fn split(&mut self, prev: Option<&BvhNode>, split_by: Axis, split_at: f32) {
        let mut left = Self::default();
        let mut left_hasher = DefaultHasher::default();

        let mut right = Self::default();
        let mut right_hasher = DefaultHasher::default();

        for triangle in mem::take(&mut self.tris) {
            let (side, hasher) = if triangle.center[split_by] <= split_at {
                (&mut left, &mut left_hasher)
            } else {
                (&mut right, &mut right_hasher)
            };

            side.add(triangle);

            OrderedFloat(triangle.center.x).hash(hasher);
            OrderedFloat(triangle.center.y).hash(hasher);
            OrderedFloat(triangle.center.z).hash(hasher);
        }

        let left_hash = left_hasher.finish();
        let right_hash = right_hasher.finish();

        if let Some(BvhNode::Internal {
            left: prev_left,
            left_hash: prev_left_hash,
            right: prev_right,
            right_hash: prev_right_hash,
            ..
        }) = prev
        {
            if *prev_left_hash == left_hash {
                left = SahBvhNode {
                    previous: Some((**prev_left).clone()),
                    ..Default::default()
                };
            } else {
                left.balance(Some(prev_left));
            }

            if *prev_right_hash == right_hash {
                right = SahBvhNode {
                    previous: Some((**prev_right).clone()),
                    ..Default::default()
                };
            } else {
                right.balance(Some(prev_right));
            }
        } else {
            left.balance(None);
            right.balance(None);
        }

        self.children =
            Some([(Box::new(left), left_hash), (Box::new(right), right_hash)]);
    }

    fn map(mut self) -> BvhNode {
        if let Some(node) = self.previous.take() {
            return node;
        }

        if let Some([left, right]) = self.children {
            BvhNode::Internal {
                bb: self.bb,
                left: Box::new(left.0.map()),
                left_hash: left.1,
                right: Box::new(right.0.map()),
                right_hash: right.1,
            }
        } else {
            BvhNode::Leaf {
                bb: self.bb,
                tris: self
                    .tris
                    .into_iter()
                    .map(|tri| (tri.triangle_id, tri.material_id))
                    .collect(),
            }
        }
    }
}
