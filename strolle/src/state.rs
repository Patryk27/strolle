use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Resource;
use bevy::math::Affine3A;
use bevy::pbr::StandardMaterial;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::{Buffer, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::CachedTexture;
use bevy::utils::HashMap;

use crate::{
    gpu, ImageHandle, InstanceHandle, Light, LightHandle, MaterialHandle,
    MeshHandle, Sun,
};

#[derive(Default, Resource)]
pub struct State {
    pub world: UniformBuffer<gpu::World>,
    pub cameras: HashMap<Entity, CameraBuffers>,
}

impl State {
    pub fn camera(&self, handle: Entity) -> &CameraBuffers {
        self.cameras
            .get(&handle)
            .unwrap_or_else(|| panic!("camera not known: {handle:?}"))
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.world.write_buffer(device, queue);

        for camera in self.cameras.values_mut() {
            camera.camera.write_buffer(device, queue);
        }
    }
}

#[derive(Component)]
pub struct CameraTextures {
    pub indirect_rays: CachedTexture,
    pub indirect_gbuffer_d0: CachedTexture,
    pub indirect_gbuffer_d1: CachedTexture,
    pub indirect_samples: CachedTexture,
    pub indirect_diffuse: CachedTexture,
}

#[derive(Default)]
pub struct CameraBuffers {
    pub camera: UniformBuffer<gpu::Camera>,
    pub indirect_samples: Option<Buffer>,
}

#[derive(Component, Debug)]
pub struct ExtractedStrolleCamera;

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

#[derive(Debug, Resource)]
pub(crate) struct ExtractedSun {
    pub sun: Option<Sun>,
}
