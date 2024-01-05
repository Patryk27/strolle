mod bvh_heatmap;
mod frame_composition;

pub use self::bvh_heatmap::*;
pub use self::frame_composition::*;

mod prelude {
    pub use std::borrow::Cow;

    pub use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
    pub use bevy::ecs::query::QueryItem;
    pub use bevy::prelude::*;
    pub use bevy::render::camera::ExtractedCamera;
    pub use bevy::render::render_graph::{
        NodeRunError, RenderGraphContext, ViewNode,
    };
    pub use bevy::render::render_resource::{
        BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry,
        CachedComputePipelineId, CachedRenderPipelineId,
        ComputePipelineDescriptor, FragmentState, PipelineCache,
        RenderPipelineDescriptor, Sampler,
    };
    pub use bevy::render::renderer::{RenderContext, RenderDevice};
    pub use bevy::render::view::ViewTarget;
    pub use wgpu::{
        BindGroupLayoutDescriptor, BindingType, BlendState, BufferBindingType,
        ColorTargetState, ColorWrites, ComputePassDescriptor, MultisampleState,
        Operations, PrimitiveState, RenderPassColorAttachment,
        RenderPassDescriptor, SamplerBindingType, SamplerDescriptor,
        ShaderStages, StorageTextureAccess, TextureSampleType,
        TextureViewDimension,
    };

    pub use crate::bvh::Bvh;
    pub use crate::camera::{CameraTextures, CamerasBuffers};
    pub use crate::images::Images;
    pub use crate::materials::Materials;
    pub use crate::triangles::Triangles;
}
