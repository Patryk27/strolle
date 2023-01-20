use std::ops::{Index, IndexMut};

use spirv_std::glam::Vec3;

use crate::bvh::{BoundingBox, BvhNode, BvhTriangle};

/// Builds BVH using SAH.
///
/// Special thanks to:
/// - https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/,
/// - https://github.com/svenstaro/bvh.
pub fn build(triangles: impl IntoIterator<Item = BvhTriangle>) -> BvhNode {
    let mut root = SahBvhNode::default();

    for triangle in triangles {
        root.add(triangle);
    }

    root.balance();
    root.map()
}

#[derive(Default)]
struct SahBvhNode {
    bb: BoundingBox,
    triangles: Vec<BvhTriangle>,
    children: Option<[Box<Self>; 2]>,
}

impl SahBvhNode {
    fn add(&mut self, triangle: BvhTriangle) {
        self.bb = self.bb + BoundingBox::from_triangle(triangle.triangle);
        self.triangles.push(triangle);
    }

    fn balance(&mut self) {
        let best = self
            .triangles
            .iter()
            .map(|triangle| triangle.triangle.center())
            .flat_map(|split_at| {
                Axis::all().map(move |split_by| (split_at, split_by))
            })
            .map(|(split_at, split_by)| {
                let splitting_cost =
                    self.estimate_splitting(split_at, split_by);

                (split_at, split_by, splitting_cost)
            })
            .min_by(|(_, _, cost_a), (_, _, cost_b)| cost_a.total_cmp(cost_b));

        if let Some((split_at, split_by, splitting_cost)) = best {
            let current_cost = (self.triangles.len() as f32) * self.bb.area();

            if splitting_cost < current_cost {
                self.split(split_at, split_by);
            }
        }
    }

    fn estimate_splitting(&self, split_at: Vec3, split_by: Axis) -> f32 {
        let mut left = 0;
        let mut left_bb = BoundingBox::default();
        let mut right = 0;
        let mut right_bb = BoundingBox::default();

        for triangle in &self.triangles {
            let (side, side_bb) =
                if triangle.triangle.center()[split_by] < split_at[split_by] {
                    (&mut left, &mut left_bb)
                } else {
                    (&mut right, &mut right_bb)
                };

            *side += 1;

            *side_bb = *side_bb + BoundingBox::from_triangle(triangle.triangle);
        }

        let cost =
            (left as f32) * left_bb.area() + (right as f32) * right_bb.area();

        cost.max(1.0)
    }

    fn split(&mut self, split_at: Vec3, split_by: Axis) {
        let mut left = Self::default();
        let mut right = Self::default();

        for triangle in self.triangles.drain(..) {
            let side =
                if triangle.triangle.center()[split_by] < split_at[split_by] {
                    &mut left
                } else {
                    &mut right
                };

            side.add(triangle);
        }

        left.balance();
        right.balance();

        self.children = Some([Box::new(left), Box::new(right)]);
    }

    fn map(mut self) -> BvhNode {
        if let Some([left, right]) = self.children {
            BvhNode::Internal {
                bb: self.bb,
                left: Box::new(left.map()),
                right: Box::new(right.map()),
            }
        } else {
            assert!(!self.triangles.is_empty());

            let node = {
                let triangle = self.triangles.remove(0);

                BvhNode::Leaf {
                    bb: self.bb,
                    triangle_id: triangle.triangle_id,
                    material_id: triangle.material_id,
                }
            };

            if self.triangles.is_empty() {
                node
            } else {
                BvhNode::Internal {
                    bb: self.bb,
                    left: Box::new(node),
                    right: Box::new(self.map()),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn all() -> impl Iterator<Item = Self> {
        [Self::X, Self::Y, Self::Z].into_iter()
    }
}

impl Index<Axis> for Vec3 {
    type Output = f32;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl IndexMut<Axis> for Vec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
            Axis::Z => &mut self.z,
        }
    }
}
