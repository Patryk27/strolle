use core::ops::ControlFlow;

use crate::*;

#[derive(Copy, Clone, Default)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    inv_direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        let direction = direction.normalize();

        Self {
            origin,
            direction,
            inv_direction: 1.0 / direction,
        }
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn hits_box_at(self, bb_min: Vec3, bb_max: Vec3) -> f32 {
        let hit_min = (bb_min - self.origin) * self.inv_direction;
        let hit_max = (bb_max - self.origin) * self.inv_direction;

        let tmin = hit_min.min(hit_max).max_element();
        let tmax = hit_min.max(hit_max).min_element();

        if tmax >= tmin && tmax >= 0.0 {
            tmin
        } else {
            f32::MAX
        }
    }

    pub fn trace(self, world: &World, stack: RayTraversingStack) -> u32 {
        let culling = Culling::Enabled;
        let dist = f32::MAX;

        let mut closest_hit = Hit::none();
        let mut closest_tri_id = 0;

        let traversed_nodes =
            self.traverse(world, stack, culling, dist, |hit, tri_id| {
                if hit.is_closer_than(closest_hit) {
                    closest_hit = hit;
                    closest_tri_id = tri_id.get() as u32 + 1;
                }

                ControlFlow::Continue(())
            });

        if debug::ENABLE_AABB {
            traversed_nodes
        } else {
            closest_tri_id
        }
    }

    pub fn hits_anything(
        self,
        world: &World,
        stack: RayTraversingStack,
        dist: f32,
    ) -> bool {
        let culling = Culling::Disabled;
        let mut hits_anything = false;

        self.traverse(world, stack, culling, dist, |hit, _| {
            if hit.dist < dist {
                hits_anything = true;
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });

        hits_anything
    }

    fn traverse(
        self,
        world: &World,
        stack: RayTraversingStack,
        culling: Culling,
        mut dist: f32,
        mut on_triangle_hit: impl FnMut(Hit, TriangleId) -> ControlFlow<()>,
    ) -> u32 {
        let mut traversed_nodes = 0;
        let mut ptr = 0;
        let stack_begins_at = (world.local_idx * 16) as usize;
        let mut stack_ptr = stack_begins_at;

        loop {
            traversed_nodes += 1;

            let meta = world.geometry_bvh.read(ptr).x.to_bits();
            let is_node = meta & 1 == 0;

            if is_node {
                let left_ptr = ptr + 2;
                let right_ptr = (meta >> 1) as usize;

                // ---

                let left_bb_min = world.geometry_bvh.read(left_ptr).yzw();
                let left_bb_max = world.geometry_bvh.read(left_ptr + 1).xyz();

                let right_bb_min = world.geometry_bvh.read(right_ptr).yzw();
                let right_bb_max = world.geometry_bvh.read(right_ptr + 1).xyz();

                // ---

                let left_dist = self.hits_box_at(left_bb_min, left_bb_max);
                let right_dist = self.hits_box_at(right_bb_min, right_bb_max);

                let near_dist = left_dist.min(right_dist);
                let far_dist = left_dist.max(right_dist);

                let (near_ptr, far_ptr) = if left_dist < right_dist {
                    (left_ptr, right_ptr)
                } else {
                    (right_ptr, left_ptr)
                };

                // ---

                if near_dist < dist {
                    ptr = near_ptr;

                    if far_dist < dist {
                        stack[stack_ptr] = far_ptr;
                        stack_ptr += 1;
                    }

                    continue;
                }
            } else {
                let tri_id = meta >> 1;
                let tri_id = TriangleId::new(tri_id as usize);
                let tri = world.geometry_tris.get(tri_id);
                let hit = tri.hit(self, culling);

                match on_triangle_hit(hit, tri_id) {
                    ControlFlow::Continue(_) => {
                        dist = dist.min(hit.dist);
                    }
                    ControlFlow::Break(_) => {
                        break;
                    }
                }
            }

            if stack_ptr > stack_begins_at {
                stack_ptr -= 1;
                ptr = stack[stack_ptr];
            } else {
                break;
            }
        }

        traversed_nodes
    }
}
