use bevy::prelude::*;
use bevy::render::renderer::RenderQueue;
use bevy::utils::HashMap;
use strolle as st;

use crate::EngineRes;

#[derive(Default, Resource)]
pub(crate) struct SyncedState {
    pub views: HashMap<Entity, SyncedView>,
}

impl SyncedState {
    pub fn is_active(&self) -> bool {
        !self.views.is_empty()
    }

    pub fn write(
        &mut self,
        engine: &mut st::Engine<EngineRes>,
        queue: &RenderQueue,
    ) {
        if !self.is_active() {
            return;
        }

        engine.write(queue);

        for view in self.views.values_mut() {
            view.viewport.write(queue);
        }
    }
}

pub struct SyncedView {
    pub viewport: st::Viewport,
}

#[derive(Resource)]
pub struct ExtractedMeshes {
    pub changed: Vec<(Handle<Mesh>, Mesh)>,
    pub removed: Vec<Handle<Mesh>>,
}

#[derive(Resource)]
pub struct ExtractedMaterials {
    pub changed: Vec<(Handle<StandardMaterial>, StandardMaterial)>,
    pub removed: Vec<Handle<StandardMaterial>>,
}

#[derive(Resource)]
pub struct ExtractedInstances {
    pub items: Vec<(Handle<Mesh>, Handle<StandardMaterial>, Mat4)>,
}

#[derive(Resource)]
pub struct ExtractedLights {
    pub items: Vec<(Entity, st::Light)>,
}

#[derive(Component)]
pub struct ExtractedCamera {
    pub transform: GlobalTransform,
    pub projection: PerspectiveProjection,
    pub clear_color: Color,
}
