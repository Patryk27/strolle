use bevy::math::Affine3A;
use bevy::prelude::*;

use crate::{
    CameraMode, ImageHandle, InstanceHandle, Light, LightHandle,
    MaterialHandle, MeshHandle, Sun,
};

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMeshes {
    pub changed: Vec<ExtractedMesh>,
    pub removed: Vec<MeshHandle>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMesh {
    pub handle: MeshHandle,
    pub mesh: Mesh,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedMaterials {
    pub changed: Vec<ExtractedMaterial>,
    pub removed: Vec<MaterialHandle>,
}

#[derive(Debug)]
pub(crate) struct ExtractedMaterial {
    pub handle: MaterialHandle,
    pub material: StandardMaterial,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedImages {
    pub changed: Vec<ExtractedImage>,
    pub removed: Vec<ImageHandle>,
}

#[derive(Debug)]
pub(crate) struct ExtractedImage {
    pub handle: ImageHandle,
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
    pub removed: Vec<InstanceHandle>,
}

#[derive(Debug)]
pub(crate) struct ExtractedInstance {
    pub handle: InstanceHandle,
    pub mesh_handle: MeshHandle,
    pub material_handle: MaterialHandle,
    pub xform: Affine3A,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedLights {
    pub changed: Vec<ExtractedLight>,
    pub removed: Vec<LightHandle>,
}

#[derive(Debug)]
pub(crate) struct ExtractedLight {
    pub handle: LightHandle,
    pub light: Light,
}

#[derive(Debug, Component)]
pub(crate) struct ExtractedCamera {
    pub transform: Mat4,
    pub projection: Mat4,
    pub mode: Option<CameraMode>,
}

#[derive(Debug, Resource)]
pub(crate) struct ExtractedSun {
    pub sun: Option<Sun>,
}
