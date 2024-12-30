use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::utils::HashMap;
use strolle as st;

use crate::EngineParams;

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
    pub handle: st::CameraHandle,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMeshes {
    pub changed: Vec<ExtractedMesh>,
    pub removed: Vec<AssetId<Mesh>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMesh {
    pub handle: AssetId<Mesh>,
    pub mesh: Mesh,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMaterials {
    pub changed: Vec<ExtractedMaterial>,
    pub removed: Vec<AssetId<StandardMaterial>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMaterial {
    pub handle: AssetId<StandardMaterial>,
    pub material: StandardMaterial,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedImages {
    pub changed: Vec<ExtractedImage>,
    pub removed: Vec<AssetId<Image>>,
}

#[derive(Debug)]
pub(crate) struct ExtractedImage {
    pub handle: AssetId<Image>,
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
pub(crate) struct ExtractedInstances {
    pub changed: Vec<ExtractedInstance>,
    pub removed: Vec<Entity>,
}

#[derive(Debug)]
pub(crate) struct ExtractedInstance {
    pub handle: Entity,
    pub mesh_handle: AssetId<Mesh>,
    pub material_handle: AssetId<StandardMaterial>,
    pub xform: Affine3A,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedLights {
    pub changed: Vec<ExtractedLight>,
    pub removed: Vec<Entity>,
}

#[derive(Debug)]
pub(crate) struct ExtractedLight {
    pub handle: Entity,
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
