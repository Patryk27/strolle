use core::mem;

use glam::Vec3;

use crate::{
    BvhNode, BvhStack, BvhView, Hit, MaterialId, TriangleId, TrianglesView,
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
        stack: BvhStack,
    ) -> (Hit, u32) {
        let mut hit = Hit::none();

        let traversed_nodes = self.trace(
            local_idx,
            triangles,
            bvh,
            stack,
            TracingMode::Nearest,
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
        stack: BvhStack,
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
        stack: BvhStack,
        mode: TracingMode,
        hit: &mut Hit,
    ) -> u32 {
        let mut traversed_nodes = 0;

        // Index into the `bvh` array; points at the currently processed node
        let mut bvh_ptr = 0;

        // Where this particular thread's stack starts at; see `BvhStack`
        let stack_begins_at = (local_idx as usize) * BVH_STACK_SIZE;

        // Index into the `stack` array; our stack spans from here up to +
        // BVH_STACK_SIZE items
        let mut stack_ptr = stack_begins_at;

        loop {
            traversed_nodes += 1;

            let (is_internal, arg0, arg1) = bvh.get(bvh_ptr).deserialize();

            if is_internal {
                let mut near_ptr = bvh_ptr + 1;

                let mut near_distance =
                    self.distance_to_node(bvh.get(near_ptr));

                let mut far_ptr = arg0;

                let mut far_distance = self.distance_to_node(bvh.get(far_ptr));

                if far_distance < near_distance {
                    mem::swap(&mut near_ptr, &mut far_ptr);
                    mem::swap(&mut near_distance, &mut far_distance);
                }

                // If the nearest child is closer than our current best shot,
                // let's check that child first; use stack to save the other
                // node for later.
                //
                // The reasoning here goes that the closer child is more likely
                // to contain a triangle we can hit; but if we don't hit that
                // triangle (kind of a "cache miss" kind of thing), we still
                // have to check the other node.
                if far_distance < hit.distance {
                    unsafe {
                        *stack.get_unchecked_mut(stack_ptr) = far_ptr;
                        stack_ptr += 1;
                    }
                }

                if near_distance < hit.distance {
                    bvh_ptr = near_ptr;
                    continue;
                }
            } else {
                let has_more_triangles = arg0 & 1 == 1;
                let triangle_id = TriangleId::new(arg0 >> 1);
                let material_id = MaterialId::new(arg1);

                if triangles.get(triangle_id).hit(self, hit) {
                    hit.material_id = material_id;

                    if let TracingMode::Any = mode {
                        break;
                    }
                }

                if has_more_triangles {
                    bvh_ptr += 1;
                    continue;
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
                unsafe {
                    stack_ptr -= 1;
                    bvh_ptr = *stack.get_unchecked(stack_ptr);
                }
            } else {
                break;
            }
        }

        traversed_nodes
    }

    /// Performs ray <-> AABB-box hit-testing and returns the closest hit (or
    /// `f32::MAX` if this ray doesn't hit given box).
    fn distance_to_node(self, node: BvhNode) -> f32 {
        let hit_min = (node.bb_min() - self.origin) * self.inv_direction;
        let hit_max = (node.bb_max() - self.origin) * self.inv_direction;

        let tmin = hit_min.min(hit_max).max_element();
        let tmax = hit_min.max(hit_max).min_element();

        if tmax >= tmin && tmax >= 0.0 {
            tmin
        } else {
            f32::INFINITY
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TracingMode {
    Nearest,
    Any,
}
