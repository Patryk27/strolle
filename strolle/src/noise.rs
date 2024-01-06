use std::mem;

use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::prelude::World;
use bevy::render::render_resource::{BufferVec, IntoBinding};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::BufferUsages;

#[derive(Resource)]
pub struct Noise {
    blue_noise_sobol: BufferVec<i32>,
    blue_noise_scrambling_tile: BufferVec<i32>,
    blue_noise_ranking_tile: BufferVec<i32>,
    dirty: bool,
}

impl Noise {
    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        if !mem::take(&mut self.dirty) {
            return;
        }

        self.blue_noise_sobol.write_buffer(device, queue);
        self.blue_noise_scrambling_tile.write_buffer(device, queue);
        self.blue_noise_ranking_tile.write_buffer(device, queue);
    }

    pub fn bind_blue_noise_sobol(&self) -> impl IntoBinding {
        self.blue_noise_sobol
            .buffer()
            .expect("buffer not ready: blue_noise_sobol")
            .as_entire_buffer_binding()
    }

    pub fn bind_blue_noise_scrambling_tile(&self) -> impl IntoBinding {
        self.blue_noise_scrambling_tile
            .buffer()
            .expect("buffer not ready: blue_noise_scrambling_tile")
            .as_entire_buffer_binding()
    }

    pub fn bind_blue_noise_ranking_tile(&self) -> impl IntoBinding {
        self.blue_noise_ranking_tile
            .buffer()
            .expect("buffer not ready: blue_noise_ranking_tile")
            .as_entire_buffer_binding()
    }
}

impl FromWorld for Noise {
    fn from_world(_: &mut World) -> Self {
        use blue_noise_sampler::spp2 as bn;

        let mut this = Self {
            blue_noise_sobol: BufferVec::new(BufferUsages::STORAGE),
            blue_noise_scrambling_tile: BufferVec::new(BufferUsages::STORAGE),
            blue_noise_ranking_tile: BufferVec::new(BufferUsages::STORAGE),
            dirty: true,
        };

        this.blue_noise_sobol
            .set_label(Some("strolle_blue_noise_sobol"));

        this.blue_noise_sobol.extend(bn::SOBOL.iter().copied());

        // ---

        this.blue_noise_scrambling_tile
            .set_label(Some("strolle_blue_noise_scrambling_tile"));

        this.blue_noise_scrambling_tile
            .extend(bn::SCRAMBLING_TILE.iter().copied());

        // ---

        this.blue_noise_ranking_tile
            .set_label(Some("strolle_blue_noise_ranking_tile"));

        this.blue_noise_ranking_tile
            .extend(bn::RANKING_TILE.iter().copied());

        this
    }
}
