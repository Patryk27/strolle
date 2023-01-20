use std::mem;
use std::ops::Range;

use spirv_std::glam::UVec2;
use strolle_models as gpu;

use crate::buffers::{
    DescriptorSet, MappedUniformBuffer, UnmappedStorageBuffer,
};
use crate::{Engine, Params};

#[derive(Debug)]
pub struct RayTracingPass {
    ds0: DescriptorSet,
    ds1: DescriptorSet,
    pipeline: wgpu::ComputePipeline,
}

impl RayTracingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: &MappedUniformBuffer<gpu::Camera>,
        ray_origins: &UnmappedStorageBuffer,
        ray_directions: &UnmappedStorageBuffer,
        ray_hits: &UnmappedStorageBuffer,
    ) -> Self
    where
        P: Params,
    {
        let ds0 = DescriptorSet::builder("strolle_ray_tracing_ds0")
            .add(&engine.triangles)
            .add(&engine.bvh)
            .add(&engine.world)
            .build(device);

        let ds1 = DescriptorSet::builder("strolle_ray_tracing_ds1")
            .add(camera)
            .add(ray_origins)
            .add(ray_directions)
            .add(ray_hits)
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_ray_tracing_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::RayPassParams>() as u32,
                    },
                }],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_ray_tracing_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.ray_tracing_pass,
                entry_point: "main",
            });

        Self { ds0, ds1, pipeline }
    }

    pub fn run(
        &self,
        size: UVec2,
        params: gpu::RayPassParams,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_ray_tracing_pass"),
            });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.ds0.bind_group(), &[]);
        pass.set_bind_group(1, self.ds1.bind_group(), &[]);
        pass.set_push_constants(0, bytemuck::bytes_of(&params));
        pass.dispatch_workgroups(size.x / 8, size.y / 8, 1);
    }
}
