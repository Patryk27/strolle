use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Index;

use crate::utils::Allocator;
use crate::{
    gpu, Bindable, BufferFlushOutcome, Images, MappedStorageBuffer, Material,
    Params,
};

#[derive(Debug)]
pub struct Materials<P>
where
    P: Params,
{
    allocator: Allocator,
    buffer: MappedStorageBuffer<Vec<gpu::Material>>,
    index: HashMap<P::MaterialHandle, gpu::MaterialId>,
    materials: Vec<Material<P>>,
}

impl<P> Materials<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            allocator: Default::default(),
            buffer: MappedStorageBuffer::new_default(device, "materials"),
            index: Default::default(),
            materials: Default::default(),
        }
    }

    pub fn insert(&mut self, handle: P::MaterialHandle, item: Material<P>) {
        match self.index.entry(handle) {
            Entry::Occupied(entry) => {
                let id = *entry.get();

                self.materials[id.get() as usize] = item;
            }

            Entry::Vacant(entry) => {
                let id = if let Some(alloc) = self.allocator.take(1) {
                    alloc.start
                } else {
                    self.materials.push(item);
                    self.materials.len() - 1
                };

                entry.insert(gpu::MaterialId::new(id as u32));
            }
        }
    }

    pub fn has(&self, handle: P::MaterialHandle) -> bool {
        self.index.contains_key(&handle)
    }

    pub fn remove(&mut self, handle: P::MaterialHandle) {
        let Some(id) = self.index.remove(&handle) else {
            return;
        };

        let id = id.get() as usize;

        self.allocator.give(id..id);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn lookup(&self, handle: P::MaterialHandle) -> Option<gpu::MaterialId> {
        self.index.get(&handle).copied()
    }

    pub fn refresh(&mut self, images: &Images<P>) {
        *self.buffer = self
            .materials
            .iter()
            .map(|material| material.serialize(images))
            .collect();
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        self.buffer.flush(device, queue)
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}

impl<P> Index<gpu::MaterialId> for Materials<P>
where
    P: Params,
{
    type Output = Material<P>;

    fn index(&self, index: gpu::MaterialId) -> &Self::Output {
        &self.materials[index.get() as usize]
    }
}
