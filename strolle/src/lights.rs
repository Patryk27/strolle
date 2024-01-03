use std::collections::hash_map::Entry;
use std::collections::HashMap;

use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::prelude::World;
use bevy::render::render_resource::BufferVec;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::BufferUsages;

use crate::{gpu, Light, LightHandle};

#[derive(Resource)]
pub struct Lights {
    buffer: BufferVec<gpu::Light>,
    index: HashMap<LightHandle, gpu::LightId>,
}

impl Lights {
    pub fn add(&mut self, handle: LightHandle, light: Light) {
        let light = light.serialize();

        match self.index.entry(handle) {
            Entry::Occupied(entry) => {
                let light_id = *entry.get();

                self.buffer.values_mut()[light_id.get() as usize] = light;
            }

            Entry::Vacant(entry) => {
                let light_id = gpu::LightId::new(self.buffer.len() as u32);

                self.buffer.push(light);
                entry.insert(light_id);
            }
        }
    }

    pub fn remove(&mut self, handle: LightHandle) {
        let Some(light_id) = self.index.remove(&handle) else {
            return;
        };

        self.buffer.values_mut().remove(light_id.get() as usize);

        for light_id2 in self.index.values_mut() {
            if light_id2.get() > light_id.get() {
                *light_id2.get_mut() -= 1;
            }
        }
    }

    pub fn update_sun(&mut self, world: gpu::World) {
        let sun_color =
            strolle_atmosphere_shader::generate_transmittance_lut::eval(
                gpu::Atmosphere::VIEW_POS,
                world.sun_direction(),
            );

        let sun_color = sun_color * gpu::Atmosphere::EXPOSURE * 0.5;

        self.buffer.values_mut()[0] =
            gpu::Light::sun(world.sun_position(), sun_color);
    }

    pub fn len(&self) -> u32 {
        self.buffer.len() as u32
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.buffer.write_buffer(device, queue);
    }
}

impl FromWorld for Lights {
    fn from_world(world: &mut World) -> Self {
        let mut buffer = BufferVec::new(BufferUsages::STORAGE);

        buffer.push(gpu::Light::sun(Default::default(), Default::default()));

        Self {
            buffer,
            index: Default::default(),
        }
    }
}
