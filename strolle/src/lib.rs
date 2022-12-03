#![feature(type_alias_impl_trait)]

mod allocated_buffer;
mod allocated_uniform;
mod geometry_indexer;

use std::num::NonZeroU32;

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
    width: u32,
    height: u32,
    pipeline: wgpu::RenderPipeline,
    ds0: DescriptorSet0,
    ds1: DescriptorSet1,
    ds2: DescriptorSet2,
    ds3: DescriptorSet3,
}

impl Strolle {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        atlas_data: &[u8],
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_spirv!(
            "../../target/shader.spv"
        ));

        let ds0 = AllocatedUniform::create(device, "ds0");
        let ds1 = AllocatedUniform::create(device, "ds1");
        let ds2 = AllocatedUniform::create(device, "ds2");

        let tex_size = wgpu::Extent3d {
            width: ATLAS_WIDTH,
            height: ATLAS_HEIGHT,
            depth_or_array_layers: 1,
        };

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas_tex"),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(ATLAS_WIDTH * 4),
                rows_per_image: NonZeroU32::new(ATLAS_HEIGHT),
            },
            tex_size,
        );

        let tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        let tex_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas_tex_sampler"),
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
                label: Some("tex_bind_group_layout"),
            });

        let ds3 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ds3"),
            layout: &ds3_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("raytracer_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                    ds2.bind_group_layout(),
                    &ds3_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("raytracer_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            width,
            height,
            pipeline,
            ds0,
            ds1,
            ds2,
            ds3,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        static_geo: &StaticGeometry,
        static_geo_index: &StaticGeometryIndex,
        dynamic_geo: &DynamicGeometry,
        uvs: &TriangleUvs,
        camera: &Camera,
        lights: &Lights,
        materials: &Materials,
        output_texture: &wgpu::TextureView,
    ) {
        self.ds0.write0(queue, static_geo);
        self.ds1.write0(queue, static_geo_index);
        self.ds1.write1(queue, dynamic_geo);
        self.ds1.write2(queue, uvs);
        self.ds2.write0(queue, camera);
        self.ds2.write1(queue, lights);
        self.ds2.write2(queue, materials);

        let mut rpass =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("raytracer_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        rpass.set_scissor_rect(0, 0, self.width as _, self.height as _);
        rpass.set_pipeline(&self.pipeline);

        rpass.set_bind_group(0, self.ds0.bind_group(), &[]);
        rpass.set_bind_group(1, self.ds1.bind_group(), &[]);
        rpass.set_bind_group(2, self.ds2.bind_group(), &[]);
        rpass.set_bind_group(3, &self.ds3, &[]);

        rpass.draw(0..3, 0..1);
    }
}
