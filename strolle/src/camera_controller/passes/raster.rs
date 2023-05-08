use std::collections::hash_map::RawEntryMut;
use std::collections::HashMap;
use std::mem;
use std::ops::Range;

use log::info;

use crate::{
    gpu, BindGroup, Bindable, Camera, CameraBuffers, CameraController, Engine,
    Params, Texture,
};

const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Depth32Float;

#[derive(Debug)]
pub struct RasterPass<P>
where
    P: Params,
{
    depth_texture: Texture,
    bg0: BindGroup,
    pipelines: HashMap<P::MaterialHandle, MaterialPipeline<P>>,
}

impl<P> RasterPass<P>
where
    P: Params,
{
    pub fn new(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        info!("Initializing pass: raster");

        let depth_texture = Texture::new(
            device,
            "strolle_raster_depth",
            camera.viewport.size,
            DEPTH_TEXTURE_FORMAT,
        );

        let bg0 = BindGroup::builder("strolle_raster_bg0")
            .add(&buffers.camera)
            .build(device);

        let pipelines = engine
            .materials
            .handles()
            .map(|material_handle| {
                let material_pipeline = MaterialPipeline::new(
                    engine,
                    device,
                    &bg0,
                    material_handle,
                );

                (material_handle.clone(), material_pipeline)
            })
            .collect();

        Self {
            depth_texture,
            bg0,
            pipelines,
        }
    }

    pub fn run(
        &self,
        engine: &Engine<P>,
        camera: &CameraController<P>,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        P: Params,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_raster_pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.primary_hits_d0.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.primary_hits_d1.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: camera.buffers.primary_hits_d2.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                },
            ),
        });

        pass.set_bind_group(0, self.bg0.as_ref(), &[]);

        for (instance_handle, instance) in engine.instances.iter() {
            let Some(material_id) = engine.materials.lookup(instance.material_handle()) else {
                continue;
            };

            let Some(pipeline) = self.pipelines.get(instance.material_handle()) else {
                continue;
            };

            let params = gpu::RasterPassParams {
                material_id: material_id.get(),
                has_normal_map: pipeline.normal_map_texture.is_some() as u32,
            };

            let (vertices, vertex_buffer) =
                engine.triangles.as_vertex_buffer(instance_handle);

            pass.set_pipeline(&pipeline.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer);
            pass.set_bind_group(1, pipeline.bg1.as_ref(), &[]);

            pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                0,
                bytemuck::bytes_of(&params),
            );

            pass.draw(0..(vertices as u32), 0..1);
        }
    }

    pub fn on_image_changed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        image_handle: &P::ImageHandle,
    ) {
        let affected_pipelines =
            self.pipelines.iter_mut().filter(|(_, material_bg)| {
                material_bg.base_color_texture.contains(image_handle)
                    || material_bg.normal_map_texture.contains(image_handle)
            });

        for (material_handle, material_bg) in affected_pipelines {
            material_bg.rebuild(engine, device, &self.bg0, material_handle);
        }
    }

    pub fn on_image_removed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        image_handle: &P::ImageHandle,
    ) {
        for (material_handle, material_bg) in self.pipelines.iter_mut() {
            let mut is_affected = false;

            if material_bg.base_color_texture.contains(image_handle) {
                material_bg.base_color_texture = None;
                is_affected = true;
            }

            if material_bg.normal_map_texture.contains(image_handle) {
                material_bg.normal_map_texture = None;
                is_affected = true;
            }

            if is_affected {
                material_bg.rebuild(engine, device, &self.bg0, material_handle);
            }
        }
    }

    pub fn on_material_changed(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        material_handle: &P::MaterialHandle,
    ) {
        match self.pipelines.raw_entry_mut().from_key(material_handle) {
            RawEntryMut::Occupied(entry) => {
                entry.into_mut().rebuild(
                    engine,
                    device,
                    &self.bg0,
                    material_handle,
                );
            }

            RawEntryMut::Vacant(entry) => {
                entry.insert(
                    material_handle.clone(),
                    MaterialPipeline::new(
                        engine,
                        device,
                        &self.bg0,
                        material_handle,
                    ),
                );
            }
        }
    }

    pub fn on_material_removed(&mut self, material_handle: &P::MaterialHandle) {
        self.pipelines.remove(material_handle);
    }
}

#[derive(Debug)]
struct MaterialPipeline<P>
where
    P: Params,
{
    bg1: BindGroup,
    pipeline: wgpu::RenderPipeline,
    base_color_texture: Option<P::ImageHandle>,
    normal_map_texture: Option<P::ImageHandle>,
}

impl<P> MaterialPipeline<P>
where
    P: Params,
{
    fn new(
        engine: &Engine<P>,
        device: &wgpu::Device,
        bg0: &BindGroup,
        material_handle: &P::MaterialHandle,
    ) -> Self {
        let base_color_texture;
        let normal_map_texture;

        if let Some(material) = engine.materials.get(material_handle) {
            base_color_texture = material.base_color_texture();
            normal_map_texture = material.normal_map_texture();
        } else {
            base_color_texture = None;
            normal_map_texture = None;
        }

        let bg1 = BindGroup::builder("stolle_raster_bg1")
            .add(&MaterialBindGroupTexture::new(engine, base_color_texture))
            .add(&MaterialBindGroupTexture::new(engine, normal_map_texture))
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_raster_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref(), bg1.as_ref()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::RasterPassParams>() as u32,
                    },
                }],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_raster_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.shaders.raster,
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
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_TEXTURE_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.shaders.raster,
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
                    ],
                }),
                multiview: None,
            });

        Self {
            bg1,
            pipeline,
            base_color_texture: base_color_texture.cloned(),
            normal_map_texture: normal_map_texture.cloned(),
        }
    }

    fn rebuild(
        &mut self,
        engine: &Engine<P>,
        device: &wgpu::Device,
        bg0: &BindGroup,
        material_handle: &P::MaterialHandle,
    ) {
        *self = Self::new(engine, device, bg0, material_handle);
    }
}

struct MaterialBindGroupTexture<'a> {
    texture: &'a wgpu::TextureView,
    sampler: &'a wgpu::Sampler,
}

impl<'a> MaterialBindGroupTexture<'a> {
    fn new<P>(
        engine: &'a Engine<P>,
        image_handle: Option<&P::ImageHandle>,
    ) -> Self
    where
        P: Params,
    {
        let (texture, sampler) = image_handle
            .and_then(|image_handle| engine.images.get(image_handle))
            .unwrap_or_else(|| engine.images.get_fallback());

        Self { texture, sampler }
    }
}

impl Bindable for MaterialBindGroupTexture<'_> {
    fn bind(
        &self,
        binding: u32,
    ) -> Vec<(wgpu::BindGroupLayoutEntry, wgpu::BindingResource)> {
        let tex_layout = wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float {
                    filterable: true, // TODO huh why
                },
            },
            count: None,
        };

        let sampler_layout = wgpu::BindGroupLayoutEntry {
            binding: binding + 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        let tex_resource = wgpu::BindingResource::TextureView(self.texture);
        let sampler_resource = wgpu::BindingResource::Sampler(self.sampler);

        vec![
            (tex_layout, tex_resource),
            (sampler_layout, sampler_resource),
        ]
    }
}
