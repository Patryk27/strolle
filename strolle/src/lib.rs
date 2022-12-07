#![feature(type_alias_impl_trait)]

mod allocated_buffer;
mod allocated_uniform;
mod geometry_indexer;

use std::sync::Arc;

pub use strolle_shader_common::*;

use self::allocated_buffer::*;
use self::allocated_uniform::*;
pub use self::geometry_indexer::*;

pub const ATLAS_WIDTH: u32 = 2048;
pub const ATLAS_HEIGHT: u32 = 512;

type DescriptorSet0 = AllocatedUniform<StaticGeometry>;
type DescriptorSet1 =
    AllocatedUniform<StaticGeometryIndex, DynamicGeometry, TriangleUvs>;
type DescriptorSet2 = AllocatedUniform<Camera, Lights, Materials>;
type DescriptorSet3 = wgpu::BindGroup;

pub struct Strolle {
    ds0: Arc<DescriptorSet0>,
    ds1: Arc<DescriptorSet1>,
    ds2: Arc<DescriptorSet2>,
    ds3: Arc<DescriptorSet3>,
    shader_module: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
}

impl Strolle {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_spirv!(
            "../../target/shader.spv"
        ));

        let ds0 = AllocatedUniform::create(device, "ds0");
        let ds1 = AllocatedUniform::create(device, "ds1");
        let ds2 = AllocatedUniform::create(device, "ds2");

        let atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("strolle_atlas_tex"),
            size: wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
        });

        let atlas_tex_view =
            atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let atlas_tex_sampler =
            device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("strolle_atlas_tex_sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: None,
                lod_min_clamp: -100.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            });

        let ds3_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("strolle_ds3_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true,
                            },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering,
                        ),
                        count: None,
                    },
                ],
            });

        let ds3 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("strolle_ds3"),
            layout: &ds3_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &atlas_tex_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(
                        &atlas_tex_sampler,
                    ),
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                    ds2.bind_group_layout(),
                    &ds3_layout,
                ],
                push_constant_ranges: &[],
            });

        Self {
            ds0: Arc::new(ds0),
            ds1: Arc::new(ds1),
            ds2: Arc::new(ds2),
            ds3: Arc::new(ds3),
            shader_module,
            pipeline_layout,
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        static_geo: &StaticGeometry,
        static_geo_index: &StaticGeometryIndex,
        dynamic_geo: &DynamicGeometry,
        uvs: &TriangleUvs,
        camera: &Camera,
        lights: &Lights,
        materials: &Materials,
    ) {
        self.ds0.write0(queue, static_geo);
        self.ds1.write0(queue, static_geo_index);
        self.ds1.write1(queue, dynamic_geo);
        self.ds1.write2(queue, uvs);
        self.ds2.write0(queue, camera);
        self.ds2.write1(queue, lights);
        self.ds2.write2(queue, materials);
    }

    pub fn create_renderer(
        &self,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> StrolleRenderer {
        log::debug!("Creating renderer (texture_format={:?})", texture_format);

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_pipeline"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader_module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        StrolleRenderer {
            ds0: self.ds0.clone(),
            ds1: self.ds1.clone(),
            ds2: self.ds2.clone(),
            ds3: self.ds3.clone(),
            pipeline,
            texture_format,
        }
    }
}

pub struct StrolleRenderer {
    ds0: Arc<DescriptorSet0>,
    ds1: Arc<DescriptorSet1>,
    ds2: Arc<DescriptorSet2>,
    ds3: Arc<DescriptorSet3>,
    pipeline: wgpu::RenderPipeline,
    texture_format: wgpu::TextureFormat,
}

impl StrolleRenderer {
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut rpass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("strolle_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Default::default()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        // TODO?
        // rpass.set_scissor_rect(0, 0, 500, 500);
        rpass.set_pipeline(&self.pipeline);

        rpass.set_bind_group(0, self.ds0.bind_group(), &[]);
        rpass.set_bind_group(1, self.ds1.bind_group(), &[]);
        rpass.set_bind_group(2, self.ds2.bind_group(), &[]);
        rpass.set_bind_group(3, &self.ds3, &[]);

        rpass.draw(0..3, 0..1);
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }
}

impl Drop for StrolleRenderer {
    fn drop(&mut self) {
        log::debug!(
            "Releasing renderer (texture_format={:?})",
            self.texture_format
        );
    }
}
