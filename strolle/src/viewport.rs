use std::mem;

use spirv_std::glam::UVec2;
use strolle_models::Camera;

use crate::buffers::{DescriptorSet, StorageBuffer, Texture, UniformBuffer};
use crate::Engine;

pub struct Viewport {
    pos: UVec2,
    size: UVec2,
    format: wgpu::TextureFormat,
    camera: UniformBuffer<Camera>,
    tracer_ds0: DescriptorSet,
    tracer_ds1: DescriptorSet,
    tracer_pipeline: wgpu::ComputePipeline,
    materializer_ds0: DescriptorSet,
    materializer_ds1: DescriptorSet,
    materializer_pipeline: wgpu::ComputePipeline,
    printer_ds0: DescriptorSet,
    printer_pipeline: wgpu::RenderPipeline,
}

impl Viewport {
    pub(crate) fn new(
        engine: &Engine,
        device: &wgpu::Device,
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
    ) -> Self {
        log::info!(
            "Creating viewport ({})",
            Viewport::describe(pos, size, format)
        );

        assert!(size.x > 0);
        assert!(size.y > 0);

        let camera = UniformBuffer::new(device, "strolle_camera");

        let hits = StorageBuffer::new(
            device,
            "strolle_hits",
            (size.x * size.y) as usize * mem::size_of::<u32>(),
        );

        let image = Texture::new(device, "strolle_image", size);

        let (tracer_ds0, tracer_ds1, tracer_pipeline) =
            Self::build_tracer(engine, device, &camera, &hits);

        let (materializer_ds0, materializer_ds1, materializer_pipeline) =
            Self::build_materializer(engine, device, &camera, &hits, &image);

        let (printer_ds0, printer_pipeline) =
            Self::build_printer(engine, device, format, &camera, &image);

        Self {
            pos,
            size,
            format,
            camera,
            tracer_ds0,
            tracer_ds1,
            tracer_pipeline,
            materializer_ds0,
            materializer_ds1,
            materializer_pipeline,
            printer_ds0,
            printer_pipeline,
        }
    }

    fn build_tracer(
        engine: &Engine,
        device: &wgpu::Device,
        camera: &UniformBuffer<Camera>,
        hits: &StorageBuffer<u32>,
    ) -> (DescriptorSet, DescriptorSet, wgpu::ComputePipeline) {
        let ds0 = DescriptorSet::builder("strolle_tracer_ds0")
            .add(&*engine.geometry_tris)
            .add(&*engine.geometry_uvs)
            .add(&*engine.geometry_bvh)
            .add(&*engine.lights)
            .add(&*engine.materials)
            .build(device);

        let ds1 = DescriptorSet::builder("strolle_tracer_ds1")
            .add(camera)
            .add(hits)
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_tracer_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_tracer_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.tracer,
                entry_point: "main",
            });

        (ds0, ds1, pipeline)
    }

    fn build_materializer(
        engine: &Engine,
        device: &wgpu::Device,
        camera: &UniformBuffer<Camera>,
        hits: &StorageBuffer<u32>,
        image: &Texture,
    ) -> (DescriptorSet, DescriptorSet, wgpu::ComputePipeline) {
        let ds0 = DescriptorSet::builder("strolle_materializer_ds0")
            .add(&*engine.geometry_tris)
            .add(&*engine.geometry_uvs)
            .add(&*engine.geometry_bvh)
            .add(&*engine.lights)
            .add(&*engine.materials)
            .build(device);

        let ds1 = DescriptorSet::builder("strolle_materializer_ds1")
            .add(camera)
            .add(hits)
            .add(&image.writable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_materializer_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_materializer_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.materializer,
                entry_point: "main",
            });

        (ds0, ds1, pipeline)
    }

    fn build_printer(
        engine: &Engine,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        camera: &UniformBuffer<Camera>,
        image: &Texture,
    ) -> (DescriptorSet, wgpu::RenderPipeline) {
        let ds0 = DescriptorSet::builder("strolle_printer_ds0")
            .add(camera)
            .add(&image.readable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_printer_pipeline_layout"),
                bind_group_layouts: &[ds0.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_printer_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.printer,
                    entry_point: "main_vs",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.printer,
                    entry_point: "main_fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        (ds0, pipeline)
    }

    pub fn pos(&self) -> UVec2 {
        self.pos
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn write(&self, queue: &wgpu::Queue, camera: &Camera) {
        self.camera.write(queue, camera);
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_tracer_pass"),
            });

        pass.set_pipeline(&self.tracer_pipeline);
        pass.set_bind_group(0, self.tracer_ds0.bind_group(), &[]);
        pass.set_bind_group(1, self.tracer_ds1.bind_group(), &[]);
        pass.dispatch_workgroups(self.size.x / 8, self.size.y / 8, 1);

        drop(pass);

        // -----

        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_materializer_pass"),
            });

        pass.set_pipeline(&self.materializer_pipeline);
        pass.set_bind_group(0, self.materializer_ds0.bind_group(), &[]);
        pass.set_bind_group(1, self.materializer_ds1.bind_group(), &[]);
        pass.dispatch_workgroups(self.size.x / 8, self.size.y / 8, 1);

        drop(pass);

        // -----

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_printer_pass"),
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

        pass.set_scissor_rect(self.pos.x, self.pos.y, self.size.x, self.size.y);
        pass.set_pipeline(&self.printer_pipeline);
        pass.set_bind_group(0, self.printer_ds0.bind_group(), &[]);
        pass.draw(0..3, 0..1);
    }

    fn describe(
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
    ) -> String {
        format!(
            "pos={}x{}, size={}x{}, format={:?}",
            pos.x, pos.y, size.x, size.y, format
        )
    }
}

impl Drop for Viewport {
    fn drop(&mut self) {
        log::info!(
            "Releasing viewport ({})",
            Self::describe(self.pos, self.size, self.format)
        );
    }
}
