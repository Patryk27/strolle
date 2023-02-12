use std::collections::HashMap;
use std::fmt::Debug;
use std::mem;

use strolle_models as gpu;

use crate::buffers::{Bindable, MappedStorageBuffer};
use crate::{Params, Triangle};

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
            buffer: MappedStorageBuffer::new_default(
                device,
                "strolle_triangles",
                (128 + 64) * 1024 * 1024,
            ),
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
        let index = self.index.get_mut(instance_handle).unwrap_or_else(|| {
            panic!("Instance not known: {instance_handle:?}")
        });

        let mut buffer = self.buffer[index.min_triangle_id..].iter_mut();

        for triangle in triangles {
            *buffer.next().unwrap() = triangle.serialize();
        }

        index.dirty = true;
        self.dirty = true;
    }

    pub fn remove(&mut self, instance_handle: &P::InstanceHandle) {
        let Some(instance) = self.index.remove(instance_handle) else { return };
        let removed_triangles = instance.triangle_count();

        for instance2 in self.index.values_mut() {
            if instance2.min_triangle_id >= instance.max_triangle_id {
                let old_indices =
                    instance2.min_triangle_id..=instance2.max_triangle_id;

                instance2.min_triangle_id -= removed_triangles;
                instance2.max_triangle_id -= removed_triangles;
                instance2.dirty = true;

                let _ = self.buffer.drain(old_indices);
            }
        }

        if let Some(max) = self
            .index
            .values()
            .map(|instance| instance.max_triangle_id)
            .max()
        {
            self.buffer.truncate(max);
        } else {
            self.buffer.clear();
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

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if !mem::take(&mut self.dirty) {
            return;
        }

        for instance in self.index.values_mut() {
            if !mem::take(&mut instance.dirty) {
                continue;
            }

            let offset =
                instance.min_triangle_id * mem::size_of::<gpu::Triangle>();

            let size =
                instance.triangle_count() * mem::size_of::<gpu::Triangle>();

            self.buffer.flush_ex(queue, offset, size);
        }
    }
}

impl<P> Bindable for Triangles<P>
where
    P: Params,
{
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        self.buffer.bind(binding)
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
