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
    has_specular: bool,
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
            has_specular: false,
        }
    }

    pub fn insert(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material<P>,
    ) {
        self.has_specular |= material.metallic > 0.0;

        match self.index.entry(material_handle) {
            Entry::Occupied(entry) => {
                let material_id = *entry.get();

                self.materials[material_id.get() as usize] = material;
            }

            Entry::Vacant(entry) => {
                let material_id =
                    if let Some(material_id) = self.allocator.take(1) {
                        material_id.start
                    } else {
                        self.materials.push(material);
                        self.materials.len() - 1
                    };

                entry.insert(gpu::MaterialId::new(material_id as u32));
            }
        }
    }

    pub fn has(&self, material_handle: &P::MaterialHandle) -> bool {
        self.index.contains_key(material_handle)
    }

    pub fn has_specular(&self) -> bool {
        self.has_specular
    }

    pub fn remove(&mut self, material_handle: &P::MaterialHandle) {
        let Some(id) = self.index.remove(material_handle) else {
            return;
        };

        let id = id.get() as usize;

        self.allocator.give(id..id);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn lookup(
        &self,
        material_handle: &P::MaterialHandle,
    ) -> Option<gpu::MaterialId> {
        self.index.get(material_handle).copied()
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
