//! This module implements a SAH-based BVH-tree builder.
//!
//! Special thanks to:
//! - https://jacco.ompf2.com/2022/04/13/how-to-build-a-bvh-part-1-basics/,
//! - https://github.com/svenstaro/bvh.

use std::fmt;
use std::ops::{Index, IndexMut};

use spirv_std::glam::Vec3;
use strolle_raytracer_models::{Triangle, TriangleId};

use super::*;
use crate::GeometryTris;

#[derive(Clone)]
pub struct SahBvh;

impl SahBvh {
    pub fn build(tris: &GeometryTris) -> BvhNode {
        let mut root = SahBvhNode::default();

        for (tri_id, tri) in tris.iter() {
            root.add(tri_id, tri);
        }

        root.balance();
        root.build()
    }
}

#[derive(Clone, Default)]
struct SahBvhNode {
    bb: BoundingBox,
    triangles: Vec<(TriangleId, Triangle)>,
    children: Option<[Box<Self>; 2]>,
}

impl SahBvhNode {
    fn add(&mut self, tri_id: TriangleId, tri: Triangle) {
        for vertex in tri.vertices() {
            self.bb.grow(vertex);
        }

        self.triangles.push((tri_id, tri));
    }

    fn balance(&mut self) {
        let mut best = None;

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            for (_, triangle) in &self.triangles {
                let cost = self.estimate_balancing_by(axis, triangle.center());

                if let Some((best_cost, best_axis, best_triangle)) = &mut best {
                    if cost < *best_cost {
                        *best_cost = cost;
                        *best_axis = axis;
                        *best_triangle = triangle;
                    }
                } else {
                    best = Some((cost, axis, triangle));
                }
            }
        }

        if let Some((cost, axis, triangle)) = best {
            let curr_cost = (self.triangles.len() as f32) * self.bb.area();

            if cost < curr_cost {
                self.balance_by(axis, triangle.center());
            }
        }
    }

    fn estimate_balancing_by(&self, axis: Axis, pos: Vec3) -> f32 {
        let mut left = 0;
        let mut left_bb = BoundingBox::default();
        let mut right = 0;
        let mut right_bb = BoundingBox::default();

        for (_, triangle) in &self.triangles {
            let (side, side_bb) = if triangle.center()[axis] < pos[axis] {
                (&mut left, &mut left_bb)
            } else {
                (&mut right, &mut right_bb)
            };

            *side += 1;

            for vertex in triangle.vertices() {
                side_bb.grow(vertex);
            }
        }

        let cost =
            (left as f32) * left_bb.area() + (right as f32) * right_bb.area();

        cost.max(1.0)
    }

    fn balance_by(&mut self, axis: Axis, pos: Vec3) {
        let mut left = Self::default();
        let mut right = Self::default();

        for (tri_id, tri) in self.triangles.drain(..) {
            let side = if tri.center()[axis] < pos[axis] {
                &mut left
            } else {
                &mut right
            };

            side.add(tri_id, tri);
        }

        left.balance();
        right.balance();

        self.children = Some([Box::new(left), Box::new(right)]);
    }

    fn build(self) -> BvhNode {
        if let Some([left, right]) = self.children {
            BvhNode::Node {
                bb: self.bb,
                left: Box::new(left.build()),
                right: Box::new(right.build()),
            }
        } else {
            BvhNode::Leaf {
                tris: self.triangles.into_iter().map(|(id, _)| id).collect(),
            }
        }
    }
}

impl fmt::Display for SahBvhNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} .. {}", self.bb.min(), self.bb.max())?;

        if let Some(children) = &self.children {
            writeln!(f)?;

            for (child_idx, child) in children.iter().enumerate() {
                if child_idx > 0 {
                    writeln!(f, "+")?;
                }

                for line in child.to_string().trim().lines() {
                    writeln!(f, "| {}", line)?;
                }
            }
        } else {
            write!(f, ", {} triangles: ", self.triangles.len())?;

            for (tri_idx, (tri_id, _)) in self.triangles.iter().enumerate() {
                if tri_idx > 0 {
                    write!(f, ", ")?;
                }

                if tri_idx > 5 {
                    write!(f, "...")?;
                    break;
                }

                write!(f, "{}", tri_id)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Axis {
    X,
    Y,
    Z,
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
