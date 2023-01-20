use std::mem;
use std::ops::Range;

use spirv_std::glam::UVec2;
use strolle_models as gpu;

use crate::buffers::{DescriptorSet, MappedUniformBuffer, Texture};
use crate::{Engine, Params};

#[derive(Debug)]
pub struct DrawingPass {
    ds0: DescriptorSet,
    pipeline: wgpu::RenderPipeline,
}

impl DrawingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera: &MappedUniformBuffer<gpu::Camera>,
        colors: &Texture,
        normals: &Texture,
        bvh_heatmap: &Texture,
    ) -> Self
    where
        P: Params,
    {
        let ds0 = DescriptorSet::builder("strolle_drawing_ds0")
            .add(camera)
            .add(&colors.readable())
            .add(&normals.readable())
            .add(&bvh_heatmap.readable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_drawing_pipeline_layout"),
                bind_group_layouts: &[ds0.bind_group_layout()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::DrawingPassParams>() as u32,
                    },
                }],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_drawing_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.shaders.drawing_pass,
                    entry_point: "main_vs",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.shaders.drawing_pass,
                    entry_point: "main_fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self { ds0, pipeline }
    }

    pub fn run(
        &self,
        pos: UVec2,
        size: UVec2,
        params: gpu::DrawingPassParams,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_drawing_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        pass.set_scissor_rect(pos.x, pos.y, size.x, size.y);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.ds0.bind_group(), &[]);
        pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            0,
            bytemuck::bytes_of(&params),
        );
        pass.draw(0..3, 0..1);
    }
}
