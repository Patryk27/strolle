use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

use bevy::ecs::world::FromWorld;
use bevy::prelude::World;
use bevy::render::render_resource::BufferVec;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::BufferUsages;

use crate::bvh::Bvh;
use crate::utils::Allocator;
use crate::{gpu, BvhPrimitive, InstanceHandle, Triangle};

pub struct Triangles {
    allocator: Allocator,
    buffer: BufferVec<gpu::Triangle>,
    index: HashMap<InstanceHandle, IndexedInstance>,
    dirty: bool,
}

impl Triangles {
    pub fn add(
        &mut self,
        bvh: &mut Bvh,
        handle: InstanceHandle,
        triangles: impl Iterator<Item = Triangle> + ExactSizeIterator,
        material_id: gpu::MaterialId,
    ) {
        assert!(
            !self.index.contains_key(&handle),
            "instance {handle:?} has been already added - now it can \
             be only updated or removed"
        );

        assert!(
            triangles.len() > 0,
            "instance {handle:?} contains no triangles"
        );

        let triangle_ids = if let Some(triangle_ids) =
            self.allocator.take(triangles.len())
        {
            self.add_reusing_space(bvh, triangles, material_id, triangle_ids)
        } else {
            self.add_allocating_space(bvh, triangles, material_id)
        };

        self.index.insert(
            handle,
            IndexedInstance {
                triangle_ids,
                dirty: true,
            },
        );

        self.dirty = true;
    }

    fn add_reusing_space(
        &mut self,
        bvh: &mut Bvh,
        triangles: impl Iterator<Item = Triangle>,
        material_id: gpu::MaterialId,
        triangle_ids: Range<usize>,
    ) -> Range<usize> {
        let mut triangle_id = triangle_ids.start;

        let iter = triangles
            .into_iter()
            .zip(&mut self.buffer.values_mut()[triangle_ids.clone()])
            .zip(bvh.update(triangle_ids.clone()));

        for ((triangle, tri), prim) in iter {
            *tri = triangle.serialize();

            *prim = BvhPrimitive {
                triangle_id: gpu::TriangleId::new(triangle_id as u32),
                material_id,
                center: triangle.center(),
                bounds: triangle.bounds(),
            };

            triangle_id += 1;
        }

        triangle_ids
    }

    fn add_allocating_space(
        &mut self,
        bvh: &mut Bvh,
        triangles: impl Iterator<Item = Triangle>,
        material_id: gpu::MaterialId,
    ) -> Range<usize> {
        let first_triangle_id = self.buffer.len();

        for (triangle_idx, triangle) in triangles.enumerate() {
            self.buffer.push(triangle.serialize());

            bvh.add(BvhPrimitive {
                triangle_id: gpu::TriangleId::new(
                    (first_triangle_id + triangle_idx) as u32,
                ),
                material_id,
                center: triangle.center(),
                bounds: triangle.bounds(),
            });
        }

        first_triangle_id..self.buffer.len()
    }

    pub fn update(
        &mut self,
        bvh: &mut Bvh,
        handle: InstanceHandle,
        triangles: impl Iterator<Item = Triangle> + ExactSizeIterator,
        material_id: gpu::MaterialId,
    ) {
        let instance = self
            .index
            .get_mut(&handle)
            .unwrap_or_else(|| panic!("instance not known: {handle:?}"));

        let iter = triangles
            .into_iter()
            .zip(&mut self.buffer.values_mut()[instance.triangle_ids.clone()])
            .zip(bvh.update(instance.triangle_ids.clone()));

        for ((triangle, tri), prim) in iter {
            *tri = triangle.serialize();

            prim.material_id = material_id;
            prim.center = triangle.center();
            prim.bounds = triangle.bounds();
        }

        instance.dirty = true;
        self.dirty = true;
    }

    pub fn remove(&mut self, bvh: &mut Bvh, handle: InstanceHandle) {
        let Some(instance) = self.index.remove(&handle) else {
            return;
        };

        self.allocator.give(instance.triangle_ids.clone());

        for prim in bvh.update(instance.triangle_ids) {
            prim.kill();
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn count(&self, handle: InstanceHandle) -> Option<usize> {
        self.index
            .get(&handle)
            .map(|instance| instance.triangle_ids.len())
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        if !mem::take(&mut self.dirty) {
            return;
        }

        self.buffer.write_buffer(device, queue);

        // let reallocated = self.buffer.reallocate(device, queue);

        // if reallocated {
        //     // Reallocating already flushes the entire buffer, so there's no
        //     // need to flush it again
        // } else {
        //     for instance in self.index.values_mut() {
        //         if !mem::take(&mut instance.dirty) {
        //             continue;
        //         }

        //         let offset = instance.triangle_ids.start
        //             * mem::size_of::<gpu::Triangle>();

        //         let size = instance.triangle_ids.len()
        //             * mem::size_of::<gpu::Triangle>();

        //         self.buffer.flush_part(queue, offset, size);
        //     }
        // }
    }
}

impl FromWorld for Triangles {
    fn from_world(world: &mut World) -> Self {
        Self {
            allocator: Default::default(),
            buffer: BufferVec::new(BufferUsages::STORAGE),
            index: Default::default(),
            dirty: Default::default(),
        }
    }
}

#[derive(Debug)]
struct IndexedInstance {
    triangle_ids: Range<usize>,
    dirty: bool,
}
