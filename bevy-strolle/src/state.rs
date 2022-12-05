mod geometry;
mod materials;

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

/// A tag-component inserted into entities that have been extracted by us.
///
/// Later, when a synchronized entity dies, this component allows us to
/// garbage-collect leftover stuff (say, when a mesh is deallocated, we have to
/// release its material etc.).
#[derive(Component)]
pub struct Synchronized;
