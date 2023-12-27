use bevy::math::Affine3A;
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

    pub fn tick(
        &mut self,
        engine: &mut st::Engine<EngineParams>,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) {
        if self.is_active() {
            engine.tick(device.wgpu_device(), queue);
        }
    }
}

#[derive(Debug)]
pub(crate) struct SyncedCamera {
    pub id: st::CameraHandle,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMeshes {
    pub changed: Vec<ExtractedMesh>,
    pub removed: Vec<AssetId<Mesh>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMesh {
    pub id: AssetId<Mesh>,
    pub mesh: Mesh,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMaterials<M>
where
    M: MaterialLike,
{
    pub changed: Vec<ExtractedMaterial<M>>,
    pub removed: Vec<AssetId<M>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMaterial<M>
where
    M: MaterialLike,
{
    pub id: AssetId<M>,
    pub material: M,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedImages {
    pub changed: Vec<ExtractedImage>,
    pub removed: Vec<AssetId<Image>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedImage {
    pub id: AssetId<Image>,
    pub texture_descriptor: wgpu::TextureDescriptor<'static>,
    pub sampler_descriptor: wgpu::SamplerDescriptor<'static>,
    pub data: ExtractedImageData,
}

#[derive(Debug)]
pub(crate) enum ExtractedImageData {
    Raw { data: Vec<u8> },
    Texture { is_dynamic: bool },
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedInstances<M>
where
    M: MaterialLike,
{
    pub changed: Vec<ExtractedInstance<M>>,
    pub removed: Vec<Entity>,
}

#[derive(Debug)]
pub(crate) struct ExtractedInstance<M>
where
    M: MaterialLike,
{
    pub id: Entity,
    pub mesh_id: AssetId<Mesh>,
    pub material_id: AssetId<M>,
    pub xform: Affine3A,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedLights {
    pub changed: Vec<ExtractedLight>,
    pub removed: Vec<Entity>,
}

#[derive(Debug)]
pub(crate) struct ExtractedLight {
    pub id: Entity,
    pub light: st::Light,
}

#[derive(Debug, Component)]
pub(crate) struct ExtractedCamera {
    pub transform: Mat4,
    pub projection: Mat4,
    pub mode: Option<st::CameraMode>,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedSun {
    pub sun: Option<st::Sun>,
}
