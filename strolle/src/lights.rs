use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{
    gpu, Bindable, BufferFlushOutcome, Light, MappedStorageBuffer, Params,
};

#[derive(Debug)]
pub struct Lights<P>
where
    P: Params,
{
    // TODO benchmark with uniform
    buffer: MappedStorageBuffer<Vec<gpu::Light>>,
    index: HashMap<P::LightHandle, gpu::LightId>,
}

impl<P> Lights<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: MappedStorageBuffer::new_default(device, "stolle_lights"),
            index: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.index.clear();
    }

    pub fn add(&mut self, light_handle: P::LightHandle, light: Light) {
        let light = light.build();

        match self.index.entry(light_handle) {
            Entry::Occupied(entry) => {
                let light_id = *entry.get();

                self.buffer[light_id.get() as usize] = light;
            }

            Entry::Vacant(entry) => {
                let light_id = gpu::LightId::new(self.buffer.len() as u32);

                self.buffer.push(light);
                entry.insert(light_id);
            }
        }
    }

    pub fn remove(&mut self, light_handle: &P::LightHandle) {
        let Some(light_id) = self.index.remove(light_handle) else { return };

        self.buffer.remove(light_id.get() as usize);

        for light_id2 in self.index.values_mut() {
            if light_id2.get() > light_id.get() {
                *light_id2 = gpu::LightId::new(light_id2.get() - 1);
            }
        }
    }

    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
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
