use std::borrow::Cow;

use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy::render::render_resource::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, CachedComputePipelineId,
    ComputePipelineDescriptor, PipelineCache,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use wgpu::{
    BindGroupLayoutDescriptor, BindingType, BufferBindingType,
    ComputePassDescriptor, ShaderStages, TextureSampleType,
};

#[derive(Resource)]
pub struct BvhHeatmapPipeline {
    layout: BindGroupLayout,
    id: CachedComputePipelineId,
}

impl FromWorld for BvhHeatmapPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("strolle_bvh_heatmap"),
                entries: &[
                    // triangles
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // bvh
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // materials
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // images
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // camera
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = {
            let shader = include_bytes!(env!("strolle_bvh_heatmap_shader.spv"));

            world
                .resource::<AssetServer>()
                .add(Shader::from_spirv(shader.to_vec(), "bvh_heatmap.spv"))
        };

        let id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::Borrowed("strolle_bvh_heatmap")),
                layout: vec![layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::Borrowed("main"),
            });

        Self { layout, id }
    }
}

#[derive(Component)]
pub struct BvhHeatmapBindGroups {
    bind_group: BindGroup,
}

#[derive(Default)]
pub struct BvhHeatmapNode;

impl ViewNode for BvhHeatmapNode {
    type ViewQuery = (&'static ExtractedCamera, &'static BvhHeatmapBindGroups);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (camera, bind_groups): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let entity = graph.view_entity();
        let pipeline = world.resource::<BvhHeatmapPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline.id)
        else {
            return Ok(());
        };

        let Some(camera_size) = camera.physical_viewport_size else {
            return Ok(());
        };

        let mut pass = render_context.command_encoder().begin_compute_pass(
            &ComputePassDescriptor {
                label: Some("strolle_bvh_heatmap_pass"),
                ..default()
            },
        );

        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_groups.bind_group, &[]);

        pass.dispatch_workgroups(
            (camera_size.x + 7) / 8,
            (camera_size.y + 7) / 8,
            1,
        );

        Ok(())
    }
}
