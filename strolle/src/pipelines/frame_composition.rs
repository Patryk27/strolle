use super::prelude::*;

#[derive(Resource)]
pub struct FrameCompositionPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    id: CachedRenderPipelineId,
}

impl FromWorld for FrameCompositionPipeline {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();

        let layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("strolle_frame_composition"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(
                            SamplerBindingType::NonFiltering,
                        ),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float {
                                filterable: false,
                            },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let sampler = device.create_sampler(&SamplerDescriptor::default());

        let shader = {
            let shader =
                include_bytes!(env!("strolle_frame_composition_shader.spv"));

            world.resource::<AssetServer>().add(Shader::from_spirv(
                shader.to_vec(),
                "frame_composition.spv",
            ))
        };

        let id = world.resource_mut::<PipelineCache>().queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("strolle_frame_composition_pipeline".into()),
                layout: vec![layout.clone()],
                push_constant_ranges: Vec::default(),
                vertex: fullscreen_shader_vertex_state(),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: Default::default(),
                    entry_point: Cow::Borrowed("main"),
                    targets: vec![Some(ColorTargetState {
                        format: ViewTarget::TEXTURE_FORMAT_HDR,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
            },
        );

        Self {
            layout,
            sampler,
            id,
        }
    }
}

#[derive(Default)]
pub struct FrameCompositionNode;

impl ViewNode for FrameCompositionNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ExtractedCamera,
        &'static CameraTextures,
    );

    fn run(
        &self,
        _: &mut RenderGraphContext,
        ctxt: &mut RenderContext,
        (target, camera, camera_tex): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<FrameCompositionPipeline>();
        let pipelines = world.resource::<PipelineCache>();

        let Some(pass_pipeline) = pipelines.get_render_pipeline(pipeline.id)
        else {
            return Ok(());
        };

        let bind_group = ctxt.render_device().create_bind_group(
            "strolle_frame_composition_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                &pipeline.sampler,
                &camera_tex.indirect_diffuse.default_view,
            )),
        );

        let mut pass = ctxt.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("strolle_frame_composition_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target.main_texture_view(),
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
        });

        pass.set_render_pipeline(pass_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        if let Some(viewport) = camera.viewport.as_ref() {
            pass.set_camera_viewport(viewport);
        }

        pass.draw(0..3, 0..1);

        Ok(())
    }
}
