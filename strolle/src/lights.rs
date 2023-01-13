use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

use strolle_models as gpu;

use crate::buffers::StorageBufferable;
use crate::Params;

#[derive(Clone, Debug)]
pub struct Lights<P>
where
    P: Params,
{
    gpu_lights: Vec<gpu::Light>,
    index: HashMap<P::LightHandle, gpu::LightId>,
}

impl<P> Lights<P>
where
    P: Params,
{
    pub fn clear(&mut self) {
        self.gpu_lights.clear();
        self.index.clear();
    }

    pub fn add(&mut self, light_handle: P::LightHandle, light: gpu::Light) {
        match self.index.entry(light_handle) {
            Entry::Occupied(entry) => {
                let light_handle = entry.key();
                let light_id = *entry.get();

                log::debug!(
                    "Light updated: {:?} ({}) => {:?}",
                    light_handle,
                    light_id.get(),
                    light
                );

                self.gpu_lights[light_id.get() as usize] = light;
            }

            Entry::Vacant(entry) => {
                // let light_handle = entry.key();
                let light_id = gpu::LightId::new(self.gpu_lights.len() as u32);

                // TODO noisy
                //
                // log::debug!(
                //     "Light added: {:?} ({}) => {:?}",
                //     light_handle,
                //     light_id.get(),
                //     light
                // );

                self.gpu_lights.push(light);
                entry.insert(light_id);
            }
        }
    }

    pub fn remove(&mut self, light_handle: &P::LightHandle) {
        let Some(light_id) = self.index.remove(light_handle) else { return };

        log::debug!("Light removed: {:?} ({})", light_handle, light_id.get());

        self.gpu_lights.remove(light_id.get() as usize);

        for light_id2 in self.index.values_mut() {
            if light_id2.get() > light_id.get() {
                log::debug!(
                    "Light relocated: {} -> {}",
                    light_id2.get(),
                    light_id2.get() - 1
                );

                *light_id2 = gpu::LightId::new(light_id2.get() - 1);
            }
        }
    }

    pub fn len(&self) -> u32 {
        self.gpu_lights.len() as u32
    }
}

impl<P> Default for Lights<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            gpu_lights: Default::default(),
            index: Default::default(),
        }
    }
}

impl<P> StorageBufferable for Lights<P>
where
    P: Params,
{
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.gpu_lights)
    }
}
