mod geometry;
mod materials;

use bevy::prelude::*;
use bevy::render::renderer::RenderQueue;
use bevy::utils::HashMap;
use strolle as st;

pub use self::geometry::*;
pub use self::materials::*;

#[derive(Default, Resource)]
pub struct SyncedState {
    pub geometry: Geometry,
    pub lights: st::Lights,
    pub materials: Materials,
    pub views: HashMap<Entity, SyncedView>,
}

impl SyncedState {
    pub fn is_active(&self) -> bool {
        !self.views.is_empty()
    }

    pub fn submit(&self, engine: &st::Engine, queue: &RenderQueue) {
        let (geometry_tris, geometry_uvs, geometry_bvh) = self.geometry.inner();

        engine.submit(
            queue.0.as_ref(),
            geometry_tris,
            geometry_uvs,
            geometry_bvh,
            &self.lights,
            self.materials.inner(),
        );

        for view in self.views.values() {
            view.viewport.submit(queue, &view.camera);
        }
    }
}

pub struct SyncedView {
    pub camera: st::Camera,
    pub viewport: st::Viewport,
}

#[derive(Component)]
pub struct ExtractedCamera {
    pub transform: GlobalTransform,
    pub projection: PerspectiveProjection,
    pub clear_color: Color,
}
