use std::mem;
use std::ops::Range;

use log::debug;

use crate::{
    gpu, BindGroup, Bindable, Camera, CameraBuffers, CameraController, Engine,
    Params, Texture,
};

#[derive(Debug)]
pub struct FrameCompositionPass {
    bg0: BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl FrameCompositionPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        debug!("Initializing pass: frame_composition");

        fn a_or_b<'a>(
            a: &'a Texture,
            b: &'a Texture,
            select_a: bool,
        ) -> impl Bindable + 'a {
            if select_a {
                a.bind_readable()
            } else {
                b.bind_readable()
            }
        }

        let bg0 = BindGroup::builder("frame_composition_bg0")
            .add(&buffers.prim_gbuffer_d0.curr().bind_readable())
            .add(&buffers.prim_gbuffer_d1.curr().bind_readable())
            .add(&a_or_b(
                &buffers.di_diff_curr_colors,
                &buffers.di_diff_samples,
                camera.mode.denoise_di_diff(),
            ))
            .add(&buffers.di_spec_samples.bind_readable())
            .add(&a_or_b(
                &buffers.gi_diff_curr_colors,
                &buffers.gi_diff_samples,
                camera.mode.denoise_gi_diff(),
            ))
            .add(&buffers.gi_spec_samples.bind_readable())
            .add(&buffers.ref_colors.bind_readable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_frame_composition_pipeline_layout"),
                bind_group_layouts: &[bg0.layout()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::FrameCompositionPassParams>()
                            as u32,
                    },
                }],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_frame_composition_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.shaders.frame_composition_vs.0,
                    entry_point: engine.shaders.frame_composition_vs.1,
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.shaders.frame_composition_fs.0,
                    entry_point: engine.shaders.frame_composition_fs.1,
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: camera.viewport.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self { bg0, pipeline }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let alternate = camera.is_alternate();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_frame_composition"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let params = gpu::FrameCompositionPassParams {
            camera_mode: camera.camera.mode.serialize(),
        };

        pass.set_scissor_rect(
            camera.camera.viewport.position.x,
            camera.camera.viewport.position.y,
            camera.camera.viewport.size.x,
            camera.camera.viewport.size.y,
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.get(alternate), &[]);
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(&params),
        );
        pass.draw(0..3, 0..1);
    }
}
