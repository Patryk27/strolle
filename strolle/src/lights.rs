use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use strolle_models::{Light, LightId};

use crate::buffers::StorageBufferable;

#[derive(Clone, Debug)]
pub struct Lights<LightHandle> {
    data: Vec<Light>,
    index: HashMap<LightHandle, LightId>,
}

impl<LightHandle> Lights<LightHandle>
where
    LightHandle: Eq + Hash + Debug,
{
    pub fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }

    pub fn add(&mut self, light_handle: LightHandle, light: Light) {
        match self.index.entry(light_handle) {
            Entry::Occupied(entry) => {
                let light_handle = entry.key();
                let light_id = *entry.get();

                log::trace!(
                    "Light updated: {:?} ({}) => {:?}",
                    light_handle,
                    light_id.get(),
                    light
                );

                self.data[light_id.get() as usize] = light;
            }

            Entry::Vacant(entry) => {
                let _light_handle = entry.key();
                let light_id = LightId::new(self.data.len() as u32);

                // TODO noisy
                // log::trace!(
                //     "Light added: {:?} ({}) => {:?}",
                //     light_handle,
                //     light_id.get(),
                //     light
                // );

                self.data.push(light);
                entry.insert(light_id);
            }
        }
    }

    pub fn remove(&mut self, light_handle: &LightHandle) {
        let Some(light_id) = self.index.remove(light_handle) else { return };

        log::trace!("Light removed: {:?} ({})", light_handle, light_id.get());

        self.data.remove(light_id.get() as usize);

        for light_id2 in self.index.values_mut() {
            if light_id2.get() > light_id.get() {
                log::trace!(
                    "Light relocated: {} -> {}",
                    light_id2.get(),
                    light_id2.get() - 1
                );

                *light_id2 = LightId::new(light_id2.get() - 1);
            }
        }
    }

    pub fn len(&self) -> u32 {
        self.data.len() as u32
    }
}

impl<LightHandle> Default for Lights<LightHandle> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            index: Default::default(),
        }
    }
}

impl<LightHandle> StorageBufferable for Lights<LightHandle> {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
