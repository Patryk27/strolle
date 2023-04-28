use glam::{Vec3, Vec4Swizzles};

use crate::{
    BvhTraversingStack, BvhView, Hit, MaterialId, TriangleId, TrianglesView,
    BVH_STACK_SIZE,
};

#[derive(Copy, Clone, Default)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    inv_direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
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

    /// Traces this ray and returns its nearest hit.
    pub fn trace_nearest(
        self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        stack: BvhTraversingStack,
    ) -> (Hit, u32) {
        let mut hit = Hit::none();

        let traversed_nodes = self.trace(
            local_idx,
            triangles,
            bvh,
            stack,
            TracingMode::Nearest,
            f32::MAX,
            &mut hit,
        );

        (hit, traversed_nodes)
    }

    /// Traces this ray and returns whether it hits anything up to the given
    /// distance.
    pub fn trace_any(
        self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        stack: BvhTraversingStack,
        max_distance: f32,
    ) -> bool {
        let mut hit = Hit {
            distance: max_distance,
            ..Hit::none()
        };

        self.trace(
            local_idx,
            triangles,
            bvh,
            stack,
            TracingMode::Any,
            max_distance,
            &mut hit,
        );

        hit.distance < max_distance
    }

    #[allow(clippy::too_many_arguments)]
    fn trace(
        self,
        local_idx: u32,
        triangles: TrianglesView,
        bvh: BvhView,
        stack: BvhTraversingStack,
        mode: TracingMode,
        mut distance: f32,
        hit: &mut Hit,
    ) -> u32 {
        let mut traversed_nodes = 0;

        // Index into the `bvh` array; points at the currently processed node
        let mut bvh_ptr = 0;

        // Where this particular thread's stack starts at; see
        // `BvhTraversingStack`
        let stack_begins_at = local_idx * (BVH_STACK_SIZE as u32);

        // Index into the `stack` array; our stack spans from here up to +
        // BVH_STACK_SIZE items
        let mut stack_ptr = stack_begins_at;

        loop {
            traversed_nodes += 1;

            let d0 = bvh.get(bvh_ptr);
            let opcode = d0.x.to_bits() & 1;
            let arg0 = d0.x.to_bits() >> 1;

            let is_internal_node = opcode == 0;

            if is_internal_node {
                let left_ptr = bvh_ptr + 2;
                let right_ptr = arg0;

                let left_dist = {
                    let bb_min = bvh.get(left_ptr).yzw();
                    let bb_max = bvh.get(left_ptr + 1).xyz();

                    self.hits_box_at(bb_min, bb_max)
                };

                let right_dist = {
                    let bb_min = bvh.get(right_ptr).yzw();
                    let bb_max = bvh.get(right_ptr + 1).xyz();

                    self.hits_box_at(bb_min, bb_max)
                };

                let near_distance = left_dist.min(right_dist);
                let far_distance = left_dist.max(right_dist);

                let (near_ptr, far_ptr) = if left_dist < right_dist {
                    (left_ptr, right_ptr)
                } else {
                    (right_ptr, left_ptr)
                };

                // Now, if the nearest child (either left or right) is closer
                // than our current best shot, check that child first; use stack
                // to save the other node for later.
                //
                // The reasoning here goes that the closer child is more likely
                // to contain a triangle we can hit; but if we don't hit that
                // triangle (kind of a "cache miss" kind of thing), we still
                // have to check the other node.
                if near_distance < distance {
                    if far_distance < distance {
                        stack[stack_ptr as usize] = far_ptr;
                        stack_ptr += 1;
                    }

                    bvh_ptr = near_ptr;
                    continue;
                }
            } else {
                let d1 = bvh.get(bvh_ptr + 1);
                let arg1 = d1.w.to_bits();

                let triangle_id = TriangleId::new(arg0);
                let material_id = MaterialId::new(arg1);

                if triangles.get(triangle_id).hit(self, hit) {
                    hit.material_id = material_id.get();

                    match mode {
                        TracingMode::Nearest => {
                            distance = distance.min(hit.distance);
                        }
                        TracingMode::Any => {
                            break;
                        }
                    }
                }
            }

            // If the control flow got here, then it means we either tested a
            // leaf-node or tested an internal-node and got a miss.
            //
            // In any case, now it's the time to pop the next node from the
            // stack and investigate it; if the stack is empty, then we've
            // tested all nodes and we can safely bail out.
            let does_stack_contain_anything = stack_ptr > stack_begins_at;

            if does_stack_contain_anything {
                stack_ptr -= 1;
                bvh_ptr = stack[stack_ptr as usize];
            } else {
                break;
            }
        }

        traversed_nodes
    }

    /// Performs ray <-> AABB-box hit-testing and returns the closest hit (or
    /// `f32::MAX` if this ray doesn't hit given box).
    fn hits_box_at(self, bb_min: Vec3, bb_max: Vec3) -> f32 {
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TracingMode {
    Nearest,
    Any,
}
