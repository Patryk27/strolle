use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use strolle_models::{Material, MaterialId};

use crate::buffers::StorageBufferable;

#[derive(Clone, Debug)]
pub struct Materials<MaterialHandle> {
    data: Vec<Material>,
    index: HashMap<MaterialHandle, MaterialId>,
}

impl<MaterialHandle> Materials<MaterialHandle>
where
    MaterialHandle: Eq + Hash + Debug,
{
    pub fn add(&mut self, material_handle: MaterialHandle, material: Material) {
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

                self.data[material_id.get() as usize] = material;
            }

            Entry::Vacant(entry) => {
                let material_handle = entry.key();
                let material_id = MaterialId::new(self.data.len() as u32);

                log::trace!(
                    "Material added: {:?} ({}) => {:?}",
                    material_handle,
                    material_id.get(),
                    material
                );

                self.data.push(material);
                entry.insert(material_id);
            }
        }
    }

    pub fn remove(&mut self, material_handle: &MaterialHandle) {
        let Some(id) = self.index.remove(material_handle) else { return };

        self.data.remove(id.get() as usize);

        for id2 in self.index.values_mut() {
            if id2.get() > id.get() {
                *id2.get_mut() -= 1;
            }
        }
    }

    pub fn lookup(
        &self,
        material_handle: &MaterialHandle,
    ) -> Option<MaterialId> {
        self.index.get(material_handle).copied()
    }
}

impl<MaterialHandle> Default for Materials<MaterialHandle> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            index: Default::default(),
        }
    }
}

impl<MaterialHandle> StorageBufferable for Materials<MaterialHandle> {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
