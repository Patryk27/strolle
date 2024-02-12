use core::mem;

use glam::{Vec3, Vec4, Vec4Swizzles};
use spirv_std::arch::IndexUnchecked;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::Sampler;

use crate::{
    BvhStack, BvhView, Material, MaterialId, MaterialsView, Tex, Triangle,
    TriangleHit, TriangleId, TrianglesView, BVH_STACK_SIZE,
};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    inv_direction: Vec3,
    length: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            inv_direction: 1.0 / direction,
            length: f32::MAX,
        }
    }

    pub fn with_length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn at(self, depth: f32) -> Vec3 {
        self.origin + self.direction * depth
    }

    /// Returns the closest opaque intersection of this ray with the world, if
    /// any.
    #[allow(clippy::too_many_arguments)]
    pub fn trace(
        self,
        local_idx: u32,
        stack: BvhStack,
        triangles: TrianglesView,
        bvh: BvhView,
        materials: MaterialsView,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
    ) -> (TriangleHit, usize) {
        let mut hit = TriangleHit::none();

        let used_memory = self.traverse(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            Tracing::ReturnClosest,
            &mut hit,
        );

        (hit, used_memory)
    }

    /// Returns whether this ray intersects with anything in the world; used for
    /// shadow rays.
    #[allow(clippy::too_many_arguments)]
    pub fn intersect(
        self,
        local_idx: u32,
        stack: BvhStack,
        triangles: TrianglesView,
        bvh: BvhView,
        materials: MaterialsView,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
    ) -> bool {
        let mut hit = TriangleHit {
            distance: self.length,
            ..TriangleHit::none()
        };

        self.traverse(
            local_idx,
            stack,
            triangles,
            bvh,
            materials,
            atlas_tex,
            atlas_sampler,
            Tracing::ReturnFirst,
            &mut hit,
        );

        hit.distance < self.length
    }

    #[allow(clippy::too_many_arguments)]
    fn traverse(
        self,
        local_idx: u32,
        stack: BvhStack,
        triangles: TrianglesView,
        bvh: BvhView,
        materials: MaterialsView,
        atlas_tex: Tex,
        atlas_sampler: &Sampler,
        tracing: Tracing,
        hit: &mut TriangleHit,
    ) -> usize {
        // An estimation of the memory used when travelling the BVH; useful for
        // debugging
        let mut used_memory = 0;

        // Index into the `bvh` array; points at the currently processed node
        let mut bvh_ptr = 0;

        // Where this particular thread's stack starts at; see `BvhStack`
        let stack_begins_at = (local_idx as usize) * BVH_STACK_SIZE;

        // Index into the `stack` array; our stack spans from here up to +
        // BVH_STACK_SIZE items
        let mut stack_ptr = stack_begins_at;

        loop {
            used_memory += mem::size_of::<Vec4>();

            let d0 = bvh.get(bvh_ptr);
            let is_internal_node = d0.w.to_bits() == 0;

            if is_internal_node {
                used_memory += 3 * mem::size_of::<Vec4>();

                let d1 = bvh.get(bvh_ptr + 1);
                let d2 = bvh.get(bvh_ptr + 2);
                let d3 = bvh.get(bvh_ptr + 3);

                let mut near_ptr = bvh_ptr + 4;
                let mut far_ptr = d1.w.to_bits();

                let mut near_distance = self.intersect_box(d0.xyz(), d1.xyz());
                let mut far_distance = self.intersect_box(d2.xyz(), d3.xyz());

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
                        *stack.index_unchecked_mut(stack_ptr) = far_ptr;
                        stack_ptr += 1;
                    }
                }

                if near_distance < hit.distance {
                    bvh_ptr = near_ptr;
                    continue;
                }
            } else {
                used_memory += mem::size_of::<Triangle>();

                let flags = d0.x.to_bits();

                // Whether there are any more triangles directly following this
                // triangle.
                //
                // This corresponds to a single BVH leaf node containing
                // multiple triangles.
                let got_more_triangles = flags & 1 == 1;

                // Whether the triangle we're looking at supports alpha
                // blending.
                //
                // If this is turned on, we have to load the triangle's material
                // and compute albedo to make sure that the part of triangle we
                // hit is actually opaque at that particular hit-point.
                let has_alpha_blending = flags & 2 == 2;

                let triangle_id = TriangleId::new(d0.y.to_bits());
                let material_id = MaterialId::new(d0.z.to_bits());

                let prev_uv = hit.uv;
                let prev_normal = hit.normal;
                let prev_distance = hit.distance;

                let mut found_hit = triangles.get(triangle_id).hit(self, hit);

                if found_hit && has_alpha_blending {
                    used_memory += mem::size_of::<Material>();
                    used_memory += mem::size_of::<Vec4>();

                    let base_color = materials.get(material_id).base_color(
                        atlas_tex,
                        atlas_sampler,
                        hit.uv,
                    );

                    if base_color.w < 1.0 {
                        found_hit = false;

                        hit.uv = prev_uv;
                        hit.normal = prev_normal;
                        hit.distance = prev_distance;
                    }
                }

                if found_hit {
                    hit.material_id = material_id;

                    if let Tracing::ReturnFirst = tracing {
                        break;
                    }
                }

                if got_more_triangles {
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
            if stack_ptr > stack_begins_at {
                unsafe {
                    stack_ptr -= 1;
                    bvh_ptr = *stack.index_unchecked(stack_ptr);
                }
            } else {
                break;
            }
        }

        if hit.is_some() {
            hit.point = self.at(hit.distance);
        }

        used_memory
    }

    /// Checks whether this ray hits given bounding-box and returns their
    /// nearest intersection distance.
    ///
    /// Thanks to:
    /// https://tavianator.com/2022/ray_box_boundary.html
    pub fn intersect_box(self, aabb_min: Vec3, aabb_max: Vec3) -> f32 {
        fn min(x: f32, y: f32) -> f32 {
            x.min(y)
        }

        fn max(x: f32, y: f32) -> f32 {
            x.max(y)
        }

        let mut tmin = 0.0;
        let mut tmax = f32::MAX;

        let t1 = (aabb_min - self.origin) * self.inv_direction;
        let t2 = (aabb_max - self.origin) * self.inv_direction;

        tmin = max(tmin, min(t1.x, t2.x));
        tmax = min(tmax, max(t1.x, t2.x));

        tmin = max(tmin, min(t1.y, t2.y));
        tmax = min(tmax, max(t1.y, t2.y));

        tmin = max(tmin, min(t1.z, t2.z));
        tmax = min(tmax, max(t1.z, t2.z));

        if tmin <= tmax {
            tmin
        } else {
            f32::MAX
        }
    }

    pub fn intersect_sphere(self, radius: f32) -> f32 {
        let b = self.origin.dot(self.direction);
        let c = self.origin.dot(self.origin) - radius * radius;

        if c > 0.0 && b > 0.0 {
            return -1.0;
        }

        let discr = b * b - c;

        if discr < 0.0 {
            -1.0
        } else if discr > b * b {
            -b + discr.sqrt()
        } else {
            -b - discr.sqrt()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tracing {
    ReturnClosest,
    ReturnFirst,
}
