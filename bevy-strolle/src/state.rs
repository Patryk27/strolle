use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::utils::{HashMap, HashSet};
use strolle as st;

use crate::{EngineParams, MaterialLike};

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
        engine: &mut st::Engine<EngineParams>,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) {
        if !self.is_active() {
            return;
        }

        engine.flush(device.wgpu_device(), queue);

        for view in self.views.values_mut() {
            view.viewport.flush(queue);
        }
    }
}

pub(crate) struct SyncedView {
    pub viewport: st::Viewport,
}

#[derive(Resource)]
pub(crate) struct ExtractedMeshes {
    pub changed: Vec<(Handle<Mesh>, Mesh)>,
    pub removed: Vec<Handle<Mesh>>,
}

#[derive(Resource)]
pub(crate) struct ExtractedImages {
    pub changed: HashSet<Handle<Image>>,
    pub removed: Vec<Handle<Image>>,
}

#[derive(Resource)]
pub(crate) struct ExtractedMaterials<M>
where
    M: MaterialLike,
{
    pub changed: Vec<(Handle<M>, M)>,
    pub removed: Vec<Handle<M>>,
}

#[derive(Resource)]
pub(crate) struct ExtractedInstances<M>
where
    M: MaterialLike,
{
    pub items: Vec<(Handle<Mesh>, Handle<M>, Mat4)>,
}

#[derive(Resource)]
pub(crate) struct ExtractedLights {
    pub items: Vec<(Entity, st::Light)>,
}

#[derive(Component)]
pub(crate) struct ExtractedCamera {
    pub transform: GlobalTransform,
    pub projection: PerspectiveProjection,
    pub clear_color: Color,
}
