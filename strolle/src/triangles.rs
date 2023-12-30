use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;
use std::ops::Range;

use crate::utils::Allocator;
use crate::{
    gpu, Bindable, BufferFlushOutcome, MappedStorageBuffer, Params,
    PrimitiveOwner,
};

#[derive(Debug)]
pub struct Triangles<P>
where
    P: Params,
{
    allocator: Allocator,
    buffer: MappedStorageBuffer<Vec<gpu::Triangle>>,
    index: HashMap<PrimitiveOwner<P>, TriangleObject>,
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

    pub fn add(
        &mut self,
        handle: PrimitiveOwner<P>,
        triangles: impl Iterator<Item = gpu::Triangle> + ExactSizeIterator,
    ) -> Range<usize> {
        assert!(
            triangles.len() > 0,
            "object {handle:?} contains no triangles"
        );

        let ids;

        if let Some(object) = self.index.get_mut(&handle) {
            ids = object.ids.clone();

            if object.len() == triangles.len() {
                let tris = triangles
                    .into_iter()
                    .zip(&mut self.buffer[object.ids.clone()]);

                for (src, dst) in tris {
                    *dst = src;
                }
            } else {
                todo!();
            }

            object.dirty = true;
        } else {
            ids = if let Some(ids) = self.allocator.take(triangles.len()) {
                let tris =
                    triangles.into_iter().zip(&mut self.buffer[ids.clone()]);

                for (src, dst) in tris {
                    *dst = src;
                }

                ids
            } else {
                let first_triangle_id = self.buffer.len();

                self.buffer.extend(triangles);

                first_triangle_id..self.buffer.len()
            };

            self.index.insert(
                handle,
                TriangleObject {
                    ids: ids.clone(),
                    dirty: true,
                },
            );
        }

        self.dirty = true;

        ids
    }

    pub fn get(
        &self,
        handle: PrimitiveOwner<P>,
    ) -> impl Iterator<Item = (gpu::TriangleId, gpu::Triangle)>
           + ExactSizeIterator
           + '_ {
        let ids = self.index[&handle].ids.clone();

        self.buffer[ids.clone()]
            .iter()
            .enumerate()
            .map(move |(id, &tri)| {
                let id = gpu::TriangleId::new((ids.start + id) as u32);

                (id, tri)
            })
    }

    pub fn copy(
        &mut self,
        src: PrimitiveOwner<P>,
        dst: PrimitiveOwner<P>,
    ) -> impl Iterator<Item = (gpu::TriangleId, &mut gpu::Triangle)>
           + ExactSizeIterator {
        let src_obj = self.index[&src].clone();

        if let Some(dst_obj) = self.index.remove(&dst) {
            self.allocator.give(dst_obj.ids);
        }

        let ids = if let Some(ids) = self.allocator.take(src_obj.len()) {
            self.index.insert(
                dst,
                TriangleObject {
                    ids: ids.clone(),
                    dirty: true,
                },
            );

            self.buffer.copy_within(src_obj.ids, ids.start);

            ids
        } else {
            let ids = self.buffer.len()..(self.buffer.len() + src_obj.len());

            self.index.insert(
                dst,
                TriangleObject {
                    ids: ids.clone(),
                    dirty: true,
                },
            );

            self.buffer.extend_from_within(src_obj.ids);

            ids
        };

        self.buffer[ids.clone()]
            .iter_mut()
            .enumerate()
            .map(move |(id, tri)| {
                let id = gpu::TriangleId::new((ids.start + id) as u32);

                (id, tri)
            })
    }

    pub fn remove(&mut self, handle: &PrimitiveOwner<P>) {
        if let Some(object) = self.index.remove(handle) {
            self.allocator.give(object.ids);
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn as_vertex_buffer(
        &self,
        handle: &PrimitiveOwner<P>,
    ) -> Option<(usize, wgpu::BufferSlice<'_>)> {
        let TriangleObject { ids, .. } = self.index.get(handle)?;
        let vertices = 3 * ids.len();

        let vertex_buffer = {
            let min = ids.start * mem::size_of::<gpu::Triangle>();
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
            // Reallocating already flushes the buffer, so in here we just need
            // to reset the dirty markers

            for instance in self.index.values_mut() {
                instance.dirty = false;
            }
        } else {
            for instance in self.index.values_mut() {
                if !mem::take(&mut instance.dirty) {
                    continue;
                }

                let offset =
                    instance.ids.start * mem::size_of::<gpu::Triangle>();

                let size = instance.ids.len() * mem::size_of::<gpu::Triangle>();

                self.buffer.flush_part(queue, offset, size);
            }
        }

        BufferFlushOutcome { reallocated }
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}

#[derive(Clone, Debug)]
struct TriangleObject {
    ids: Range<usize>,
    dirty: bool,
}

impl TriangleObject {
    fn len(&self) -> usize {
        self.ids.len()
    }
}
