use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Resource;
use bevy::render::render_resource::UniformBuffer;
use bevy::render::texture::CachedTexture;
use bevy::utils::HashMap;

use crate::gpu;

#[derive(Default, Resource)]
pub struct CamerasBuffers {
    pub cameras: HashMap<Entity, CameraBuffers>,
}

pub struct CameraBuffers {
    pub camera: UniformBuffer<gpu::Camera>,
}

#[derive(Component)]
pub struct CameraTextures {
    pub indirect_diffuse: CachedTexture,
}

#[derive(Component, Debug)]
pub struct ExtractedStrolleCamera;
