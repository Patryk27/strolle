use std::mem;
use std::ops::Range;

use rand::Rng;

use crate::{gpu, BindGroup, CameraBuffers, CameraController, Engine, Params};

#[derive(Debug)]
pub struct VoxelPaintingPass {
    bg0: BindGroup,
    pipeline: wgpu::ComputePipeline,
}

impl VoxelPaintingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        log::info!("Initializing pass: voxel-painting");

        let bg0 = BindGroup::builder("strolle_voxel_painting_bg0")
            .add(&buffers.camera)
            .add(&buffers.voxels.as_rw_bind())
            .add(&buffers.pending_voxels.as_ro_bind())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_voxel_painting_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::VoxelPaintingPassParams>()
                            as u32,
                    },
                }],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_voxel_painting_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.voxel_painting,
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
                label: Some("strolle_voxel_painting_pass"),
            });

        let params = gpu::VoxelPaintingPassParams {
            frame: camera.frame,
            seed: rand::thread_rng().gen(),
        };

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.as_ref(), &[]);
        pass.set_push_constants(0, bytemuck::bytes_of(&params));
        pass.dispatch_workgroups(1, 1, 1);
    }
}
