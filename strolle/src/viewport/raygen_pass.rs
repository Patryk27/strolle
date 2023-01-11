use spirv_std::glam::UVec2;
use strolle_models as gpu;

use crate::buffers::{DescriptorSet, StorageBuffer, UniformBuffer};
use crate::{Engine, Params};

pub struct RaygenPass {
    ds0: DescriptorSet,
    pipeline: wgpu::ComputePipeline,
}

impl RaygenPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: &UniformBuffer<gpu::Camera>,
        rays: &StorageBuffer<f32>,
    ) -> Self
    where
        P: Params,
    {
        let ds0 = DescriptorSet::builder("strolle_raygen_ds0")
            .add(camera)
            .add(rays)
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_raygen_pipeline_layout"),
                bind_group_layouts: &[ds0.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_raygen_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.raygen_pass,
                entry_point: "main",
            });

        Self { ds0, pipeline }
    }

    pub fn run(&self, size: UVec2, encoder: &mut wgpu::CommandEncoder) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_raygen_pass"),
            });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.ds0.bind_group(), &[]);
        pass.dispatch_workgroups(size.x / 8, size.y / 8, 1);
    }
}
