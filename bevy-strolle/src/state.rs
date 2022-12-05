mod geometry;
mod materials;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::renderer::RenderQueue;
use strolle as st;

pub use self::geometry::*;
pub use self::materials::*;

#[derive(Default, Resource)]
pub struct ExtractedState {
    pub geometry: Geometry,
    pub camera: st::Camera,
    pub lights: st::Lights,
    pub materials: Materials,
    pub clear_color: ClearColorConfig,
}

impl ExtractedState {
    pub fn enqueue(&mut self, strolle: &st::Strolle, queue: &RenderQueue) {
        let Some((
            static_geo,
            static_geo_index,
            dynamic_geo,
            uvs,
        )) = self.geometry.inner() else { return };

        strolle.enqueue(
            queue.0.as_ref(),
            static_geo,
            static_geo_index,
            dynamic_geo,
            uvs,
            &self.camera,
            &self.lights,
            self.materials.inner(),
        );
    }
}
