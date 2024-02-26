use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use derivative::Derivative;

use crate::{
    gpu, Bindable, BufferFlushOutcome, Light, MappedStorageBuffer, Params,
};

#[derive(Debug)]
pub struct Lights<P>
where
    P: Params,
{
    buffer: MappedStorageBuffer<Vec<gpu::Light>>,
    index: HashMap<LightHandle<P>, gpu::LightId>,
    created: HashSet<LightHandle<P>>,
    updated: HashSet<LightHandle<P>>,
    remapped: HashMap<LightHandle<P>, gpu::LightId>,
    killed: HashSet<gpu::LightId>,
    next_light_id: gpu::LightId,
}

impl<P> Lights<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        let mut buffer =
            MappedStorageBuffer::<Vec<_>>::new_default(device, "stolle_lights");

        buffer.push(gpu::Light::sun(Default::default(), Default::default()));

        // ---

        let mut index = HashMap::new();

        index.insert(LightHandle::Sun, gpu::LightId::new(0));

        // ---

        Self {
            buffer,
            index,
            created: Default::default(),
            updated: Default::default(),
            remapped: Default::default(),
            killed: Default::default(),
            next_light_id: gpu::LightId::new(1),
        }
    }

    pub fn insert(&mut self, handle: P::LightHandle, item: Light) {
        let item = item.serialize();
        let handle = LightHandle::Light(handle);

        match self.index.entry(handle) {
            Entry::Occupied(entry) => {
                let id = *entry.get();

                self.update(id.get() as usize, handle, item);
            }

            Entry::Vacant(entry) => {
                if let Some(slot) =
                    self.buffer.get_mut(self.next_light_id.get() as usize)
                {
                    *slot = item;
                    entry.insert(self.next_light_id);
                } else {
                    let id = gpu::LightId::new(self.buffer.len() as u32);

                    self.buffer.push(item);
                    entry.insert(id);
                }

                self.created.insert(handle);
                *self.next_light_id.get_mut() += 1;
            }
        }
    }

    pub fn update_sun(&mut self, world: gpu::World) {
        let color =
            strolle_shaders::atmosphere::generate_transmittance_lut::eval(
                gpu::Atmosphere::VIEW_POS,
                world.sun_dir(),
            );

        // TODO probably incorrect
        let color = color * gpu::Atmosphere::EXPOSURE * 5.0;

        self.update(
            0,
            LightHandle::Sun,
            gpu::Light::sun(world.sun_pos(), color),
        );
    }

    pub fn remove(&mut self, handle: P::LightHandle) {
        let handle = LightHandle::Light(handle);

        let Some(id) = self.index.remove(&handle) else {
            return;
        };

        let idx = id.get() as usize;

        self.buffer.remove(idx);
        self.buffer.push(Default::default());

        self.created.remove(&handle);
        self.updated.remove(&handle);
        self.remapped.remove(&handle);
        self.killed.insert(id);

        *self.next_light_id.get_mut() -= 1;

        for (other_handle, other_id) in self.index.iter_mut() {
            if other_id.get() > id.get() {
                self.remapped.entry(*other_handle).or_insert(*other_id);

                *other_id.get_mut() -= 1;
            }
        }
    }

    pub fn len(&self) -> u32 {
        self.next_light_id.get()
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        for id in &self.killed {
            self.buffer[id.get() as usize].kill_slot();
        }

        for (handle, id) in &self.remapped {
            self.buffer[id.get() as usize].remap_slot(self.index[&handle]);
        }

        let outcome = self.buffer.flush(device, queue);

        for handle in self.created.iter().chain(&self.updated) {
            self.buffer[self.index[handle].get() as usize].commit();
        }

        for id in self.killed.iter().chain(self.remapped.values()) {
            self.buffer[id.get() as usize].clear_slot();
        }

        self.created.clear();
        self.updated.clear();
        self.remapped.clear();
        self.killed.clear();

        outcome
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }

    fn update(
        &mut self,
        idx: usize,
        handle: LightHandle<P>,
        mut new: gpu::Light,
    ) {
        let old = self.buffer[idx];

        new.prev_d0 = old.d0;
        new.prev_d1 = old.d1;
        new.prev_d2 = old.d2;

        self.updated.insert(handle);
        self.buffer[idx] = new;
    }
}

// TODO sun should be handled outside of Strolle, there's no reason to
//      special-case it here
#[derive(Debug, Derivative)]
#[derivative(Clone, Copy, PartialEq, Eq, Hash)]
enum LightHandle<P>
where
    P: Params,
{
    Sun,
    Light(P::LightHandle),
}
