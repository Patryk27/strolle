use log::info;

use crate::{BindGroup, CameraBuffers, CameraController, Engine, Params};

#[derive(Debug)]
pub struct RayTracingPass {
    bg0: BindGroup,
    bg1: BindGroup,
    pipeline: wgpu::ComputePipeline,
}

impl RayTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        info!("Initializing pass: ray-tracing");

        let bg0 = BindGroup::builder("strolle_ray_tracing_bg0")
            .add(&engine.triangles.as_ro_bind())
            .add(&engine.bvh.as_ro_bind())
            .add(&engine.world)
            .build(device);

        let bg1 = BindGroup::builder("strolle_ray_tracing_bg1")
            .add(&buffers.camera)
            .add(&buffers.directs.as_rw_storage_bind())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_ray_tracing_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref(), bg1.as_ref()],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_ray_tracing_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.ray_tracing,
                entry_point: "main",
            });

        Self { bg0, bg1, pipeline }
    }

    pub fn run<P>(
        &self,
        camera: &CameraController<P>,
        encoder: &mut wgpu::CommandEncoder,
    ) where
        P: Params,
    {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_ray_tracing_pass"),
            });

        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.as_ref(), &[]);
        pass.set_bind_group(1, self.bg1.as_ref(), &[]);
        pass.dispatch_workgroups(size.x, size.y, 1);
    }
}
