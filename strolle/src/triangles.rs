use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;

use crate::{
    gpu, Bindable, BufferFlushOutcome, MappedStorageBuffer, Params, Triangle,
};

#[derive(Debug)]
pub struct Triangles<P>
where
    P: Params,
{
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
            buffer: MappedStorageBuffer::new_default(device, "triangles"),
            index: Default::default(),
            dirty: Default::default(),
        }
    }

    pub fn add(
        &mut self,
        instance_handle: P::InstanceHandle,
        triangles: impl IntoIterator<Item = Triangle>,
    ) {
        assert!(
            !self.index.contains_key(&instance_handle),
            "Instance {instance_handle:?} has been already added - now it can \
             be only updated or removed"
        );

        let min_triangle_id = self.buffer.len();

        self.buffer
            .extend(triangles.into_iter().map(|triangle| triangle.serialize()));

        let max_triangle_id = self.buffer.len() - 1;

        assert!(
            max_triangle_id > min_triangle_id,
            "Instance {instance_handle:?} contains no triangles"
        );

        self.index.insert(
            instance_handle,
            IndexedInstance {
                min_triangle_id,
                max_triangle_id,
                dirty: true,
            },
        );

        self.dirty = true;
    }

    pub fn update(
        &mut self,
        instance_handle: &P::InstanceHandle,
        triangles: impl IntoIterator<Item = Triangle>,
    ) {
        let instance =
            self.index.get_mut(instance_handle).unwrap_or_else(|| {
                panic!("Instance not known: {instance_handle:?}")
            });

        let mut buffer = self.buffer[instance.min_triangle_id..].iter_mut();

        for triangle in triangles {
            *buffer.next().unwrap() = triangle.serialize();
        }

        instance.dirty = true;
        self.dirty = true;
    }

    pub fn remove(&mut self, instance_handle: &P::InstanceHandle) {
        let Some(instance) = self.index.remove(instance_handle) else { return };
        let removed_triangles = instance.triangle_count();

        let _ = self
            .buffer
            .drain(instance.min_triangle_id..=instance.max_triangle_id);

        for instance2 in self.index.values_mut() {
            if instance2.min_triangle_id >= instance.max_triangle_id {
                instance2.min_triangle_id -= removed_triangles;
                instance2.max_triangle_id -= removed_triangles;
                instance2.dirty = true;
            }
        }

        self.dirty = true;
    }

    pub fn count(&self, instance_handle: &P::InstanceHandle) -> Option<usize> {
        self.index
            .get(instance_handle)
            .map(|instance| instance.triangle_count())
    }

    pub fn iter(
        &self,
        instance_handle: &P::InstanceHandle,
    ) -> impl Iterator<Item = (gpu::TriangleId, gpu::Triangle)> + Clone + '_
    {
        self.index
            .get(instance_handle)
            .into_iter()
            .flat_map(move |instance| {
                let triangle_ids =
                    instance.min_triangle_id..=instance.max_triangle_id;

                triangle_ids.map(move |triangle_id| {
                    (
                        gpu::TriangleId::new(triangle_id as u32),
                        self.buffer[triangle_id],
                    )
                })
            })
    }

    pub fn as_vertex_buffer(
        &self,
        instance_handle: &P::InstanceHandle,
    ) -> (usize, wgpu::BufferSlice<'_>) {
        let IndexedInstance {
            min_triangle_id,
            max_triangle_id,
            ..
        } = &self.index[instance_handle];

        let vertices = 3 * (max_triangle_id - min_triangle_id + 1);

        let vertex_buffer = {
            let min = min_triangle_id * mem::size_of::<gpu::Triangle>();
            let min = min as wgpu::BufferAddress;

            // N.B. we could slice up to some `max`, but GPUs care only about
            // the start of the buffer and the number of vertices
            self.buffer.as_buffer().slice(min..)
        };

        (vertices, vertex_buffer)
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

                let offset =
                    instance.min_triangle_id * mem::size_of::<gpu::Triangle>();

                let size =
                    instance.triangle_count() * mem::size_of::<gpu::Triangle>();

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
    min_triangle_id: usize,
    max_triangle_id: usize,
    dirty: bool,
}

impl IndexedInstance {
    fn triangle_count(&self) -> usize {
        self.max_triangle_id - self.min_triangle_id + 1
    }
}
