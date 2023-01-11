use spirv_std::glam::UVec2;
use strolle_models as gpu;

use crate::buffers::{DescriptorSet, StorageBuffer, Texture, UniformBuffer};
use crate::{Engine, Params};

pub struct ShadingPass {
    ds0: DescriptorSet,
    ds1: DescriptorSet,
    pipeline: wgpu::ComputePipeline,
}

impl ShadingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        camera: &UniformBuffer<gpu::Camera>,
        rays: &StorageBuffer<f32>,
        image: &Texture,
    ) -> Self
    where
        P: Params,
    {
        let ds0 = DescriptorSet::builder("strolle_shading_ds0")
            .add(&engine.triangles)
            .add(&engine.instances)
            .add(&engine.bvh)
            .add(&engine.lights)
            .add(&engine.materials)
            .add(&engine.images.binder())
            .add(&engine.info)
            .build(device);

        let ds1 = DescriptorSet::builder("strolle_shading_ds1")
            .add(camera)
            .add(rays)
            .add(&image.writable())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_shading_pipeline_layout"),
                bind_group_layouts: &[
                    ds0.bind_group_layout(),
                    ds1.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_shading_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.shading_pass,
                entry_point: "main",
            });

        Self { ds0, ds1, pipeline }
    }

    pub fn run(&self, size: UVec2, encoder: &mut wgpu::CommandEncoder) {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_shading_pass"),
            });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.ds0.bind_group(), &[]);
        pass.set_bind_group(1, self.ds1.bind_group(), &[]);
        pass.dispatch_workgroups(size.x / 8, size.y / 8, 1);
    }
}
