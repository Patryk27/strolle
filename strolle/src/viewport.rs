use spirv_std::glam::UVec2;
use strolle_raytracer_models::Camera;
use strolle_renderer_models::Params;

use crate::buffers::{DescriptorSet, Texture, UniformBuffer};
use crate::Engine;

pub struct Viewport {
    pos: UVec2,
    size: UVec2,
    format: wgpu::TextureFormat,
    camera: UniformBuffer<Camera>,
    raytracer_ds0: DescriptorSet,
    raytracer_ds1: DescriptorSet,
    raytracer_ds2: DescriptorSet,
    raytracer_pipeline: wgpu::ComputePipeline,
    renderer_params: UniformBuffer<Params>,
    renderer_ds0: DescriptorSet,
    renderer_pipeline: wgpu::RenderPipeline,
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

        let (
            camera,
            image,
            raytracer_ds0,
            raytracer_ds1,
            raytracer_ds2,
            raytracer_pipeline,
        ) = Self::build_raytracer(engine, device, size);

        let (renderer_params, renderer_ds0, renderer_pipeline) =
            Self::build_renderer(engine, device, format, &image);

        Self {
            pos,
            size,
            format,
            camera,
            raytracer_ds0,
            raytracer_ds1,
            raytracer_ds2,
            raytracer_pipeline,
            renderer_params,
            renderer_ds0,
            renderer_pipeline,
        }
    }

    fn build_raytracer(
        engine: &Engine,
        device: &wgpu::Device,
        size: UVec2,
    ) -> (
        UniformBuffer<Camera>,
        Texture,
        DescriptorSet,
        DescriptorSet,
        DescriptorSet,
        wgpu::ComputePipeline,
    ) {
        let camera = UniformBuffer::new(device, "strolle_camera");
        let image = Texture::new(device, "strolle_image", size);

        let ds0 = DescriptorSet::builder("strolle_raytracer_ds0")
            .add(&*engine.geometry_tris)
            .add(&*engine.geometry_uvs)
            .add(&*engine.geometry_bvh)
            .build(device);

        let ds1 = DescriptorSet::builder("strolle_raytracer_ds1")
            .add(&camera)
            .add(&*engine.lights)
            .add(&*engine.materials)
            .build(device);

        let ds2 = DescriptorSet::builder("strolle_raytracer_ds2")
            .add(&image.writable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_raytracer_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                    ds2.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_raytracer_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.raytracer,
                entry_point: "main",
            });

        (camera, image, ds0, ds1, ds2, pipeline)
    }

    fn build_renderer(
        engine: &Engine,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        image: &Texture,
    ) -> (UniformBuffer<Params>, DescriptorSet, wgpu::RenderPipeline) {
        let params = UniformBuffer::new(device, "strolle_renderer_params");

        let ds0 = DescriptorSet::builder("strolle_renderer_ds0")
            .add(&params)
            .add(&image.readable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_renderer_pipeline_layout"),
                bind_group_layouts: &[ds0.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("strolle_renderer_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &engine.renderer,
                    entry_point: "main_vs",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &engine.renderer,
                    entry_point: "main_fs",
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        (params, ds0, pipeline)
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

    pub fn submit(&self, queue: &wgpu::Queue, camera: &Camera) {
        self.camera.write(queue, camera);

        self.renderer_params.write(
            queue,
            &Params {
                x: self.pos.x as f32,
                y: self.pos.y as f32,
                w: self.size.x as f32,
                h: self.size.y as f32,
            },
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_raytracer_pass"),
            });

        pass.set_pipeline(&self.raytracer_pipeline);
        pass.set_bind_group(0, self.raytracer_ds0.bind_group(), &[]);
        pass.set_bind_group(1, self.raytracer_ds1.bind_group(), &[]);
        pass.set_bind_group(2, self.raytracer_ds2.bind_group(), &[]);
        pass.dispatch_workgroups(self.size.x / 8, self.size.y / 8, 1);

        drop(pass);

        // -----

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("strolle_renderer_pass"),
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
        pass.set_pipeline(&self.renderer_pipeline);
        pass.set_bind_group(0, self.renderer_ds0.bind_group(), &[]);
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
