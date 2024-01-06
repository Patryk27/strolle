mod bvh_heatmap;
mod frame_composition;
mod indirect_shading;
mod indirect_tracing;

pub use self::bvh_heatmap::*;
pub use self::frame_composition::*;
pub use self::indirect_shading::*;
pub use self::indirect_tracing::*;

mod prelude {
    pub use std::borrow::Cow;
    pub use std::mem;

    pub use bevy::asset::AssetServer;
    pub use bevy::core::FrameCount;
    pub use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
    pub use bevy::core_pipeline::prepass::ViewPrepassTextures;
    pub use bevy::ecs::query::QueryItem;
    pub use bevy::ecs::system::Resource;
    pub use bevy::ecs::world::{FromWorld, World};
    pub use bevy::prelude::default;
    pub use bevy::render::camera::ExtractedCamera;
    pub use bevy::render::render_graph::{
        NodeRunError, RenderGraphContext, ViewNode,
    };
    pub use bevy::render::render_resource::{
        BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry,
        CachedComputePipelineId, CachedRenderPipelineId,
        ComputePipelineDescriptor, FragmentState, PipelineCache,
        RenderPipelineDescriptor, Sampler, Shader,
    };
    pub use bevy::render::renderer::{RenderContext, RenderDevice};
    pub use bevy::render::view::ViewTarget;
    pub use wgpu::{
        BindGroupLayoutDescriptor, BindingType, BlendState, BufferBindingType,
        ColorTargetState, ColorWrites, ComputePassDescriptor, MultisampleState,
        Operations, PrimitiveState, PushConstantRange,
        RenderPassColorAttachment, RenderPassDescriptor, SamplerBindingType,
        SamplerDescriptor, ShaderStages, StorageTextureAccess, TextureFormat,
        TextureSampleType, TextureViewDimension,
    };

    pub(crate) use crate::bvh::Bvh;
    pub(crate) use crate::gpu;
    pub(crate) use crate::images::Images;
    pub(crate) use crate::lights::Lights;
    pub(crate) use crate::materials::Materials;
    pub(crate) use crate::noise::Noise;
    pub(crate) use crate::state::{CameraTextures, State};
    pub(crate) use crate::triangles::Triangles;
}
