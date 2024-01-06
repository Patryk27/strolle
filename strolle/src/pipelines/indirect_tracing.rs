use super::prelude::*;

#[derive(Resource)]
pub struct IndirectTracingPipeline {
    bg0: BindGroupLayout,
    bg1: BindGroupLayout,
    sampler: Sampler,
    id: CachedComputePipelineId,
}

impl FromWorld for IndirectTracingPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let bg0 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("strolle_indirect_tracing_bg0"),
            entries: &[
                // blue_noise_sobol
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
                // blue_noise_scrambling_tile
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
                // blue_noise_ranking_tile
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
                // triangles
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
                // bvh
                BindGroupLayoutEntry {
                    binding: 4,
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
                    binding: 5,
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
                    binding: 6,
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
                    binding: 7,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bg1 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("strolle_indirect_tracing_bg1"),
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
                // bevy_gbuffer_sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // bevy_gbuffer_normals
                BindGroupLayoutEntry {
                    binding: 2,
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
                // bevy_gbuffer_depth
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
                // indirect_rays
                BindGroupLayoutEntry {
                    binding: 4,
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
                    binding: 5,
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
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let shader = {
            let shader =
                include_bytes!(env!("strolle_indirect_tracing_shader.spv"));

            world.resource::<AssetServer>().add(Shader::from_spirv(
                shader.to_vec(),
                "indirect_tracing.spv",
            ))
        };

        let id = world
            .resource_mut::<PipelineCache>()
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::Borrowed("strolle_indirect_tracing")),
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

        Self {
            bg0,
            bg1,
            sampler,
            id,
        }
    }
}

#[derive(Default)]
pub struct IndirectTracingNode;

impl ViewNode for IndirectTracingNode {
    type ViewQuery = (
        &'static ViewPrepassTextures,
        &'static ExtractedCamera,
        &'static CameraTextures,
    );

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        ctxt: &mut RenderContext,
        (prepass, camera, camera_tex): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<IndirectTracingPipeline>();
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
            "strolle_indirect_tracing_bg0",
            &pipeline.bg0,
            &BindGroupEntries::sequential((
                world.resource::<Noise>().bind_blue_noise_sobol(),
                world.resource::<Noise>().bind_blue_noise_scrambling_tile(),
                world.resource::<Noise>().bind_blue_noise_ranking_tile(),
                world.resource::<Triangles>().bind(),
                world.resource::<Bvh>().bind(),
                world.resource::<Materials>().bind(),
                world.resource::<Images>().bind_atlas_tex(),
                world.resource::<Images>().bind_atlas_sampler(),
            )),
        );

        let bg1 = ctxt.render_device().create_bind_group(
            "strolle_indirect_tracing_bg1",
            &pipeline.bg1,
            &BindGroupEntries::sequential((
                state.camera(graph.view_entity()),
                &pipeline.sampler,
                &prepass.normal.as_ref().unwrap().default_view,
                &prepass.depth.as_ref().unwrap().default_view,
                &camera_tex.indirect_rays.default_view,
                &camera_tex.indirect_gbuffer_d0.default_view,
                &camera_tex.indirect_gbuffer_d1.default_view,
            )),
        );

        let mut pass =
            ctxt.command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("strolle_indirect_tracing_pass"),
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
