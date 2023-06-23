use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::utils::HashMap;
use strolle as st;

use crate::{EngineParams, MaterialLike};

#[derive(Default, Resource)]
pub(crate) struct SyncedState {
    pub cameras: HashMap<Entity, SyncedCamera>,
}

impl SyncedState {
    pub fn is_active(&self) -> bool {
        !self.cameras.is_empty()
    }

    pub fn flush(
        &mut self,
        engine: &mut st::Engine<EngineParams>,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) {
        if self.is_active() {
            engine.flush(device.wgpu_device(), queue);
        }
    }
}

pub(crate) struct SyncedCamera {
    pub handle: st::CameraHandle,
}

#[derive(Resource)]
pub(crate) struct ExtractedMeshes {
    pub changed: Vec<(Handle<Mesh>, Mesh)>,
    pub removed: Vec<Handle<Mesh>>,
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
pub(crate) struct ExtractedImages {
    pub changed: Vec<(Handle<Image>, Image)>,
    pub removed: Vec<Handle<Image>>,
}

#[derive(Resource)]
pub(crate) struct ExtractedInstances<M>
where
    M: MaterialLike,
{
    pub changed: Vec<(Entity, Handle<Mesh>, Handle<M>, Mat4)>,
    pub removed: Vec<Entity>,
}

#[derive(Resource)]
pub(crate) struct ExtractedLights {
    pub items: Vec<(Entity, st::Light)>,
}

#[derive(Component)]
pub(crate) struct ExtractedCamera {
    pub transform: GlobalTransform,
    pub projection: PerspectiveProjection,
    pub mode: Option<st::CameraMode>,
}

#[derive(Resource)]
pub(crate) struct ExtractedSun {
    pub sun: Option<st::Sun>,
}
