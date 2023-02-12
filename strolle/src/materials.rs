use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

use strolle_models as gpu;

use crate::buffers::{Bindable, MappedStorageBuffer};
use crate::images::Images;
use crate::{Material, Params};

#[derive(Debug)]
pub struct Materials<P>
where
    P: Params,
{
    // TODO benchmark with uniform
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
            buffer: MappedStorageBuffer::new_default(
                device,
                "strolle_materials",
                4 * 1024 * 1024,
            ),
            index: Default::default(),
            materials: Default::default(),
        }
    }

    pub fn add(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material<P>,
    ) {
        match self.index.entry(material_handle) {
            Entry::Occupied(entry) => {
                let material_handle = entry.key();
                let material_id = *entry.get();

                log::debug!(
                    "Material updated: {:?} ({}) => {:?}",
                    material_handle,
                    material_id.get(),
                    material
                );

                self.materials[material_id.get() as usize] = material;
            }

            Entry::Vacant(entry) => {
                let material_handle = entry.key();
                let material_id =
                    gpu::MaterialId::new(self.materials.len() as u32);

                log::debug!(
                    "Material added: {:?} ({}) => {:?}",
                    material_handle,
                    material_id.get(),
                    material
                );

                self.materials.push(material);
                entry.insert(material_id);
            }
        }
    }

    pub fn remove(&mut self, material_handle: &P::MaterialHandle) {
        let Some(id) = self.index.remove(material_handle) else { return };

        self.materials.remove(id.get() as usize);

        for id2 in self.index.values_mut() {
            if id2.get() > id.get() {
                *id2.get_mut() -= 1;
            }
        }
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
            .map(|material| material.build(images))
            .collect();
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        self.buffer.flush(queue);
    }
}

impl<P> Bindable for Materials<P>
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
