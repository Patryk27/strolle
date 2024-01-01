use std::mem;
use std::ops::Range;

use glam::vec4;
use log::debug;

use crate::{
    gpu, BindGroup, Camera, CameraBuffers, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectRasterPass {
    bg0: BindGroup,
    bg1: BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl DirectRasterPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        debug!("Initializing pass: direct_raster");

        let bg0 = BindGroup::builder("direct_raster_bg0")
            .add(&engine.materials.bind_readable())
            .add(&engine.images.bind_atlas())
            .build(device);

        let bg1 = BindGroup::builder("direct_raster_bg1")
            .add(&buffers.camera.bind_readable())
            .add(&buffers.prev_camera.bind_readable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_direct_raster_pipeline_layout"),
                bind_group_layouts: &[bg0.layout(), bg1.layout()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::DirectRasterPassParams>()
                            as u32,
                    },
                }],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_direct_raster_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.shaders.direct_raster,
                    entry_point: "main_vs",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: (3 * 4 * mem::size_of::<f32>()) as _,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            // position (xyz) + uv (x)
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // normal (xyz) + uv (y)
                            wgpu::VertexAttribute {
                                offset: (4 * mem::size_of::<f32>()) as _,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // tangent (xyzw)
                            wgpu::VertexAttribute {
                                offset: (8 * mem::size_of::<f32>()) as _,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.shaders.direct_raster,
                    entry_point: "main_fs",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba32Float,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                multiview: None,
            });

        Self { bg0, bg1, pipeline }
    }

    pub fn run<P>(
        &self,
        engine: &Engine<P>,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        P: Params,
    {
        let alternate = camera.is_alternate();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_direct_raster"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.direct_gbuffer_d0.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.direct_gbuffer_d1.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.surface_map.get(alternate).view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.velocity_map.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: camera.buffers.direct_depth.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                },
            ),
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.get(alternate), &[]);
        pass.set_bind_group(1, self.bg1.get(alternate), &[]);

        for (instance_handle, instance_entry) in engine.instances.iter() {
            let instance = &instance_entry.instance;

            let Some(material_id) =
                engine.materials.lookup(&instance.material_handle)
            else {
                continue;
            };

            let params = {
                let curr_xform_inv = gpu::DirectRasterPassParams::encode_affine(
                    instance.xform_inv,
                );

                let prev_xform = gpu::DirectRasterPassParams::encode_affine(
                    instance_entry.prev_xform,
                );

                gpu::DirectRasterPassParams {
                    payload: vec4(
                        f32::from_bits(material_id.get()),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    ),
                    curr_xform_inv_d0: curr_xform_inv[0],
                    curr_xform_inv_d1: curr_xform_inv[1],
                    curr_xform_inv_d2: curr_xform_inv[2],
                    prev_xform_d0: prev_xform[0],
                    prev_xform_d1: prev_xform[1],
                    prev_xform_d2: prev_xform[2],
                }
            };

            let Some((vertices, vertex_buffer)) =
                engine.triangles.as_vertex_buffer(instance_handle)
            else {
                continue;
            };

            pass.set_vertex_buffer(0, vertex_buffer);

            pass.set_push_constants(
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );

            pass.draw(0..(vertices as u32), 0..1);
        }
    }
}
