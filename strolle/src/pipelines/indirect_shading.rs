use super::prelude::*;

#[derive(Resource)]
pub struct IndirectShadingPipeline {
    bg0: BindGroupLayout,
    bg1: BindGroupLayout,
    id: CachedComputePipelineId,
}

impl FromWorld for IndirectShadingPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let bg0 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("strolle_indirect_shading_bg0"),
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
                // lights
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
                // materials
                BindGroupLayoutEntry {
                    binding: 3,
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
                    binding: 4,
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
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // world
                BindGroupLayoutEntry {
                    binding: 6,
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

        let bg1 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("strolle_indirect_shading_bg1"),
            entries: &[
                // camera
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // indirect_rays
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // indirect_gbuffer_d0
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // indirect_gbuffer_d1
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // indirect_samples
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let shader = {
            let shader =
                include_bytes!(env!("strolle_indirect_shading_shader.spv"));

            world.resource::<AssetServer>().add(Shader::from_spirv(
                shader.to_vec(),
                "indirect_shading.spv",
            ))
        };

        let id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::Borrowed("strolle_indirect_shading")),
                layout: vec![bg0.clone(), bg1.clone()],
                push_constant_ranges: vec![PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: 0..(mem::size_of::<gpu::IndirectPassParams>()
                        as u32),
                }],
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::Borrowed("main"),
            });

        Self { bg0, bg1, id }
    }
}

#[derive(Default)]
pub struct IndirectShadingNode;

impl ViewNode for IndirectShadingNode {
    type ViewQuery = (&'static ExtractedCamera, &'static CameraTextures);

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        ctxt: &mut RenderContext,
        (camera, camera_tex): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<IndirectShadingPipeline>();
        let pipelines = world.resource::<PipelineCache>();

        let Some(pass_pipeline) = pipelines.get_compute_pipeline(pipeline.id)
        else {
            return Ok(());
        };

        let Some(camera_size) = camera.physical_viewport_size else {
            return Ok(());
        };

        let state = world.resource::<State>();

        let bg0 = ctxt.render_device().create_bind_group(
            "strolle_indirect_shading_bg0",
            &pipeline.bg0,
            &BindGroupEntries::sequential((
                world.resource::<Triangles>().bind(),
                world.resource::<Bvh>().bind(),
                world.resource::<Lights>().bind(),
                world.resource::<Materials>().bind(),
                world.resource::<Images>().bind_atlas_tex(),
                world.resource::<Images>().bind_atlas_sampler(),
                &state.world,
            )),
        );

        let bg1 = ctxt.render_device().create_bind_group(
            "strolle_indirect_shading_bg1",
            &pipeline.bg1,
            &BindGroupEntries::sequential((
                state.camera(graph.view_entity()),
                &camera_tex.indirect_rays.default_view,
                &camera_tex.indirect_gbuffer_d0.default_view,
                &camera_tex.indirect_gbuffer_d1.default_view,
                &camera_tex.indirect_samples.default_view,
            )),
        );

        let mut pass =
            ctxt.command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("strolle_indirect_shading_pass"),
                    ..default()
                });

        pass.set_pipeline(pass_pipeline);
        pass.set_bind_group(0, &bg0, &[]);
        pass.set_bind_group(1, &bg1, &[]);

        pass.set_push_constants(
            0,
            bytemuck::bytes_of(&gpu::IndirectPassParams {
                seed: 0,
                frame: world.resource::<FrameCount>().0,
                mode: gpu::IndirectPassParams::MODE_DIFFUSE,
            }),
        );

        pass.dispatch_workgroups(
            (camera_size.x + 7) / 8,
            (camera_size.y + 7) / 8,
            1,
        );

        Ok(())
    }
}
