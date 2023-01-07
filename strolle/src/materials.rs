use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

use strolle_models as gpu;

use crate::buffers::StorageBufferable;
use crate::images::Images;
use crate::{Material, Params};

#[derive(Clone, Debug)]
pub struct Materials<P>
where
    P: Params,
{
    cpu_materials: Vec<Material<P>>,
    gpu_materials: Vec<gpu::Material>,
    index: HashMap<P::MaterialHandle, gpu::MaterialId>,
}

impl<P> Materials<P>
where
    P: Params,
{
    pub fn add(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material<P>,
    ) {
        match self.index.entry(material_handle) {
            Entry::Occupied(entry) => {
                let material_handle = entry.key();
                let material_id = *entry.get();

                log::trace!(
                    "Material updated: {:?} ({}) => {:?}",
                    material_handle,
                    material_id.get(),
                    material
                );

                self.cpu_materials[material_id.get() as usize] = material;
            }

            Entry::Vacant(entry) => {
                let material_handle = entry.key();
                let material_id =
                    gpu::MaterialId::new(self.cpu_materials.len() as u32);

                log::trace!(
                    "Material added: {:?} ({}) => {:?}",
                    material_handle,
                    material_id.get(),
                    material
                );

                self.cpu_materials.push(material);
                entry.insert(material_id);
            }
        }
    }

    pub fn remove(&mut self, material_handle: &P::MaterialHandle) {
        let Some(id) = self.index.remove(material_handle) else { return };

        self.cpu_materials.remove(id.get() as usize);

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

    pub fn rebuild(&mut self, images: &Images<P>) {
        log::trace!("Rebuilding materials");

        self.gpu_materials = self
            .cpu_materials
            .iter()
            .map(|material| material.build(images))
            .collect();
    }
}

impl<P> Default for Materials<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            cpu_materials: Default::default(),
            gpu_materials: Default::default(),
            index: Default::default(),
        }
    }
}

impl<P> StorageBufferable for Materials<P>
where
    P: Params,
{
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.gpu_materials)
    }
}
