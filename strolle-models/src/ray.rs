use core::ops::ControlFlow;

use glam::{Mat4, Vec3, Vec4Swizzles};

use crate::{debug, BvhTraversingStack, InstanceId, TriangleId, World};

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

    /// Returns this ray's origin (in world-space or mesh-space, depending on
    /// the context).
    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    /// Returns this ray's direction (in world-space or mesh-space, depending on
    /// the context).
    ///
    /// Note that if this ray follows mesh-space, then the value returned here
    /// will not be normalized (see: [`Ray::with_transform()`]).
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Transforms this ray from world-space into mesh-space.
    ///
    /// We're calling this function when moving from world-bvh into mesh-bvh,
    /// with the transform here corresponding to the inverse transform of that
    /// particular mesh *instance*.
    ///
    /// This allows us to keep one BVH for each mesh and use inverse transforms
    /// to convert between world-bvh and mesh-bvh.
    ///
    /// Thanks to <https://jacco.ompf2.com/2022/05/07/how-to-build-a-bvh-part-5-tlas-blas>.
    fn with_transform(mut self, mat: Mat4) -> Self {
        self.origin = mat.transform_point3(self.origin);

        // Note that we deliberately don't normalize this vector - thanks to
        // this (seemigly wrong, but actually correct and pretty cool) trick,
        // the hit-tests performed across different mesh-bvhs will report
        // distances in world-space instead of mesh-space.
        //
        // If we normalized the direction in here, then when traversing the
        // tree, we couldn't compare hit-distances from different mesh-bvhs
        // (since each mesh-bvh would return distance specific to this
        // particular mesh instead of reporting it in world-space).
        self.direction = mat.transform_vector3(self.direction);

        self.inv_direction = 1.0 / self.direction;
        self
    }

    /// Follows this ray and returns the closest object it hits.
    ///
    /// This function returns a tuple over `(instance-id, triangle-id)` which
    /// is later read during the shading pass.
    ///
    /// Note that in principle this function returns `Option<...>` - to avoid
    /// having extra memory allocations, the `None` variant is encoded as
    /// `.0 == u32::MAX`.
    pub fn trace(self, world: &World, stack: BvhTraversingStack) -> (u32, u32) {
        let dist = f32::MAX;

        let mut closest_distance = f32::MAX;
        let mut closest_instance_id = u32::MAX;
        let mut closest_triangle_id = u32::MAX;

        let traversed_nodes = self.traverse(
            world,
            stack,
            dist,
            |instance_id, triangle_id, distance| {
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_instance_id = instance_id.get();
                    closest_triangle_id = triangle_id.get();
                }

                ControlFlow::Continue(())
            },
        );

        if debug::ENABLE_AABB {
            (traversed_nodes, 0)
        } else {
            (closest_instance_id, closest_triangle_id)
        }
    }

    /// Follows this ray and returns whether it hits anything up to given
    /// maximum distance; we're using this function to compute shadows.
    pub fn hits_anything(
        self,
        world: &World,
        stack: BvhTraversingStack,
        max_distance: f32,
    ) -> bool {
        let mut hits_anything = false;

        self.traverse(world, stack, max_distance, |_, _, distance| {
            if distance < max_distance {
                hits_anything = true;

                // We don't care which particular triangle is the closest one,
                // so we can bail out as soon as we prove that anything blocks
                // this ray's line of sight
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });

        hits_anything
    }

    /// Traverses the world and invokes given callback for each triangle that
    /// hits this ray.
    ///
    /// Note that since shaders can't use recursive functions, the algorithm
    /// here works on a manual stack - see: [`BvhTraversingStack`].
    fn traverse(
        mut self,
        world: &World,
        stack: BvhTraversingStack,
        mut distance: f32,
        mut on_triangle_hit: impl FnMut(
            InstanceId,
            TriangleId,
            f32,
        ) -> ControlFlow<()>,
    ) -> u32 {
        // A special value indicating, upon popping it from the stack, that we
        // have to roll-back from mesh-bvh coordinate system back into
        // world-bvh.
        //
        // This value must be distinguishable from regular bvh-ptrs and so a
        // good choice could be `1` (since all bvh-ptrs are normally even), but
        // `u32::MAX` was chosen for clarity & less hackiness.
        const INSTANCE_MARKER: u32 = u32::MAX;

        // Number of traversed BVH nodes - used for debugging purposes
        let mut traversed_nodes = 0;

        // Index into the `world.bvh` array - points at the currently-processed
        // node; we start at the world-bvh's root node
        let mut bvh_ptr = world.info.world_bvh_ptr;

        // Where this particular thread's stack starts at.
        //
        // For performance reasons, the entire workgroup shares the same stack
        // through workgroup-memory - this means that we can't start from
        // `stack[0]`, but rather have to use the current thread's index and
        // multiply it by the number of stack-slots for each thread.
        let stack_begins_at = world.local_idx * 32;

        // Index into the `stack` array; our stack spans from here up to + 32
        // items
        let mut stack_ptr = stack_begins_at;

        // Previous origin & direction; we store here ray's origin and direction
        // when we move from world-bvh into mesh-bvh and then use these
        // variables to restore the ray's parameters when going from mesh-bvh
        // back into world-bvh.
        //
        // (an alternative would be to store the instance's transformation
        // matrix, but that would require even more memory.)
        let mut prev_origin = Vec3::default();
        let mut prev_direction = Vec3::default();

        // Currently-processed instance id; technically `Option<InstanceId>`,
        // since we start from world-bvh's root which doesn't belong to any
        // instance
        let mut instance_id = InstanceId::new(0);

        // Offset into the `world.triangles` array for the currently-processed
        // instance; as above, it's technically `Option<u32>`
        let mut instance_min_triangle_id = 0;

        loop {
            traversed_nodes += 1;

            let opcode = world.bvh.get(bvh_ptr).x.to_bits();
            let is_internal = opcode & 1 == 0;
            let args = opcode >> 1;

            // If the current node is an internal-node
            if is_internal {
                // ... then its left child is going to be located right after it
                // (hence `+ 2` here, referring to this node's size)
                let left_ptr = bvh_ptr + 2;

                // ... and its right child is going to be extra `+ args` items
                // further from here (since mesh-bvh uses relative addressing)
                let right_ptr = bvh_ptr + 2 + args;

                // Load the left and right child's bounding boxes
                let left_bb_min = world.bvh.get(left_ptr).yzw();
                let left_bb_max = world.bvh.get(left_ptr + 1).xyz();
                let right_bb_min = world.bvh.get(right_ptr).yzw();
                let right_bb_max = world.bvh.get(right_ptr + 1).xyz();

                // ... compute distances to both
                let left_dist = self.hits_box_at(left_bb_min, left_bb_max);
                let right_dist = self.hits_box_at(right_bb_min, right_bb_max);

                // ... and select the closest one
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
                    bvh_ptr = near_ptr;

                    if far_distance < distance {
                        stack[stack_ptr as usize] = far_ptr;
                        stack_ptr += 1;
                    }

                    continue;
                }
            } else {
                // Otherwise, if our current node is a leaf-node
                let is_instance = args & 1 == 0;
                let args = args >> 1;

                // If it's an instance-leaf-node (i.e. mesh-bvh)
                if is_instance {
                    // ... then load its instance's details
                    instance_id = InstanceId::new(args);

                    let instance = world.instances.get(instance_id);

                    // Store our current origin & direction for rolling-back
                    // later.
                    //
                    // (due to the recursive nature of this algorithm, usually
                    // those values would have to be pushed onto our stack -
                    // fortunately for us, mesh-bvhs cannot be nested, so just
                    // keeping those values around in variables will do.)
                    prev_origin = self.origin;
                    prev_direction = self.direction;

                    // Transform our ray from world-space into mesh-space
                    self = self.with_transform(instance.inv_transform());

                    // Note down the first triangle id of this instance's mesh.
                    //
                    // (we keep all triangles from all meshes in a common buffer
                    // and so we have to note down the first id here to later
                    // convert relative triangle index - which is what mesh-bvh
                    // contains - into absolute triangle index.)
                    instance_min_triangle_id = instance.min_triangle_id().get();

                    // Push into a stack a special value that indicates we've
                    // went from world-space into mesh-space - later, upon
                    // popping it, we'll restore origin and direction
                    stack[stack_ptr as usize] = INSTANCE_MARKER;
                    stack_ptr += 1;

                    // Jump to mesh-bvh
                    bvh_ptr = instance.bvh_ptr().get();
                    continue;
                } else {
                    // If it's an instance-triangle-node (i.e. mesh-bvh's
                    // triangle)
                    let triangle_id =
                        TriangleId::new(instance_min_triangle_id + args);

                    let hit = world.triangles.get(triangle_id).hit(self);

                    match on_triangle_hit(
                        instance_id,
                        triangle_id,
                        hit.distance,
                    ) {
                        ControlFlow::Continue(_) => {
                            distance = distance.min(hit.distance);
                        }
                        ControlFlow::Break(_) => {
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

                if bvh_ptr == INSTANCE_MARKER {
                    self.origin = prev_origin;
                    self.direction = prev_direction;
                    self.inv_direction = 1.0 / self.direction;

                    // TODO optimization idea: make sure that's true already
                    //      when building the tree
                    if stack_ptr > stack_begins_at {
                        stack_ptr -= 1;
                        bvh_ptr = stack[stack_ptr as usize];
                    } else {
                        break;
                    }
                }
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
