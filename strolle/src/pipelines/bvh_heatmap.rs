use super::prelude::*;

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
                    // atlas_tex
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // atlas_sampler
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Sampler(
                            SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                    // camera
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // output
                    BindGroupLayoutEntry {
                        binding: 6,
                        visibility: ShaderStages::all(),
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: ViewTarget::TEXTURE_FORMAT_HDR,
                            view_dimension: TextureViewDimension::D2,
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

#[derive(Default)]
pub struct BvhHeatmapNode;

impl ViewNode for BvhHeatmapNode {
    type ViewQuery = (&'static ExtractedCamera, &'static CameraTextures);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        ctxt: &mut RenderContext,
        (camera, camera_tex): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<BvhHeatmapPipeline>();
        let pipelines = world.resource::<PipelineCache>();

        let Some(pass_pipeline) = pipelines.get_compute_pipeline(pipeline.id)
        else {
            return Ok(());
        };

        let Some(camera_size) = camera.physical_viewport_size else {
            return Ok(());
        };

        if world.resource::<Triangles>().is_empty() {
            return Ok(());
        }

        let buffers =
            &world.resource::<CamerasBuffers>().cameras[&graph.view_entity()];

        let bind_group = ctxt.render_device().create_bind_group(
            "strolle_bvh_heatmap_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                world.resource::<Triangles>().bind(),
                world.resource::<Bvh>().bind(),
                world.resource::<Materials>().bind(),
                world.resource::<Images>().bind_atlas_tex(),
                world.resource::<Images>().bind_atlas_sampler(),
                &buffers.camera,
                &camera_tex.indirect_diffuse.default_view,
            )),
        );

        let mut pass =
            ctxt.command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("strolle_bvh_heatmap_pass"),
                    ..default()
                });

        pass.set_pipeline(pass_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        pass.dispatch_workgroups(
            (camera_size.x + 7) / 8,
            (camera_size.y + 7) / 8,
            1,
        );

        Ok(())
    }
}
