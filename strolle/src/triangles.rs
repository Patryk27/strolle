use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

use crate::bvh::Bvh;
use crate::utils::Allocator;
use crate::{
    gpu, Bindable, BufferFlushOutcome, BvhPrimitive, MappedStorageBuffer,
    Params, Triangle,
};

#[derive(Debug)]
pub struct Triangles<P>
where
    P: Params,
{
    allocator: Allocator,
    buffer: MappedStorageBuffer<Vec<gpu::Triangle>>,
    index: HashMap<P::InstanceHandle, IndexedInstance>,
    dirty: bool,
}

impl<P> Triangles<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            allocator: Default::default(),
            buffer: MappedStorageBuffer::new_default(device, "triangles"),
            index: Default::default(),
            dirty: Default::default(),
        }
    }

    pub fn create(
        &mut self,
        bvh: &mut Bvh,
        instance_handle: P::InstanceHandle,
        triangles: impl Iterator<Item = Triangle> + ExactSizeIterator,
        material_id: gpu::MaterialId,
    ) {
        assert!(
            !self.index.contains_key(&instance_handle),
            "instance {instance_handle:?} has been already added - now it can \
             be only updated or removed"
        );

        assert!(
            triangles.len() > 0,
            "instance {instance_handle:?} contains no triangles"
        );

        let triangle_ids = if let Some(triangle_ids) =
            self.allocator.take(triangles.len())
        {
            self.create_reusing_space(bvh, triangles, material_id, triangle_ids)
        } else {
            self.create_allocating_space(bvh, triangles, material_id)
        };

        self.index.insert(
            instance_handle,
            IndexedInstance {
                triangle_ids,
                dirty: true,
            },
        );

        self.dirty = true;
    }

    fn create_reusing_space(
        &mut self,
        bvh: &mut Bvh,
        triangles: impl Iterator<Item = Triangle>,
        material_id: gpu::MaterialId,
        triangle_ids: Range<usize>,
    ) -> Range<usize> {
        let mut triangle_id = triangle_ids.start;

        let iter = triangles
            .into_iter()
            .zip(&mut self.buffer[triangle_ids.clone()])
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

    fn create_allocating_space(
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
        instance_handle: &P::InstanceHandle,
        triangles: impl Iterator<Item = Triangle> + ExactSizeIterator,
        material_id: gpu::MaterialId,
    ) {
        let instance =
            self.index.get_mut(instance_handle).unwrap_or_else(|| {
                panic!("instance not known: {instance_handle:?}")
            });

        let iter = triangles
            .into_iter()
            .zip(&mut self.buffer[instance.triangle_ids.clone()])
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

    pub fn remove(
        &mut self,
        bvh: &mut Bvh,
        instance_handle: &P::InstanceHandle,
    ) {
        let Some(instance) = self.index.remove(instance_handle) else {
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

    pub fn count(&self, instance_handle: &P::InstanceHandle) -> Option<usize> {
        self.index
            .get(instance_handle)
            .map(|instance| instance.triangle_ids.len())
    }

    pub fn as_vertex_buffer(
        &self,
        instance_handle: &P::InstanceHandle,
    ) -> Option<(usize, wgpu::BufferSlice<'_>)> {
        let IndexedInstance { triangle_ids, .. } =
            self.index.get(instance_handle)?;

        let vertices = 3 * triangle_ids.len();

        let vertex_buffer = {
            let min = triangle_ids.start * mem::size_of::<gpu::Triangle>();
            let min = min as wgpu::BufferAddress;

            // N.B. we could slice up to some `max`, but GPUs care only about
            // the start of the buffer and the number of vertices
            self.buffer.as_buffer().slice(min..)
        };

        Some((vertices, vertex_buffer))
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        if !mem::take(&mut self.dirty) {
            return BufferFlushOutcome::default();
        }

        let reallocated = self.buffer.reallocate(device, queue);

        if reallocated {
            // Reallocating already flushes the entire buffer, so there's no
            // need to flush it again
        } else {
            for instance in self.index.values_mut() {
                if !mem::take(&mut instance.dirty) {
                    continue;
                }

                let offset = instance.triangle_ids.start
                    * mem::size_of::<gpu::Triangle>();

                let size = instance.triangle_ids.len()
                    * mem::size_of::<gpu::Triangle>();

                self.buffer.flush_part(queue, offset, size);
            }
        }

        BufferFlushOutcome { reallocated }
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}

#[derive(Debug)]
struct IndexedInstance {
    triangle_ids: Range<usize>,
    dirty: bool,
}
