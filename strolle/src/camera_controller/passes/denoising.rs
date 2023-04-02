use std::mem;
use std::ops::Range;

use rand::Rng;

use crate::{gpu, BindGroup, CameraBuffers, CameraController, Engine, Params};

#[derive(Debug)]
pub struct DenoisingPass {
    bg0: BindGroup,
    pipeline: wgpu::ComputePipeline,
}

impl DenoisingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        log::info!("Initializing pass: denoising");

        let bg0 = BindGroup::builder("strolle_denoising_bg0")
            .add(&buffers.directs.as_rw_storage_bind())
            .add(&buffers.pending_directs.as_rw_storage_bind())
            .add(&buffers.indirects.as_rw_storage_bind())
            .add(&buffers.pending_indirects.as_rw_storage_bind())
            .add(&buffers.normals.as_rw_storage_bind())
            .add(&buffers.pending_normals.as_rw_storage_bind())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_denoising_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::DenoisingPassParams>() as u32,
                    },
                }],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_denoising_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.denoising,
                entry_point: "main",
            });

        Self { bg0, pipeline }
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
                label: Some("strolle_denoising_pass"),
            });

        let params = gpu::DenoisingPassParams {
            seed: rand::thread_rng().gen(),
        };

        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.as_ref(), &[]);
        pass.set_push_constants(0, bytemuck::bytes_of(&params));
        pass.dispatch_workgroups(size.x, size.y, 1);
    }
}
