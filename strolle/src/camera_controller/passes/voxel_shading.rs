use std::mem;
use std::ops::Range;

use log::info;

use crate::{gpu, BindGroup, CameraBuffers, CameraController, Engine, Params};

#[derive(Debug)]
pub struct VoxelShadingPass {
    bg0: BindGroup,
    bg1: BindGroup,
    pipeline: wgpu::ComputePipeline,
}

impl VoxelShadingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        info!("Initializing pass: voxel-shading");

        let bg0 = BindGroup::builder("strolle_voxel_shading_bg0")
            .add(&engine.triangles.as_ro_bind())
            .add(&engine.bvh.as_ro_bind())
            .add(&engine.lights.as_ro_bind())
            .add(&engine.materials.as_ro_bind())
            .add(&engine.images.as_bind())
            .add(&engine.world)
            .build(device);

        let bg1 = BindGroup::builder("strolle_voxel_shading_bg1")
            .add(&buffers.camera)
            .add(&buffers.primary_hits_d0.as_rw_storage_bind()) // TODO doesn't have to be writable
            .add(&buffers.primary_hits_d1.as_rw_storage_bind()) // TODO doesn't have to be writable
            .add(&buffers.primary_hits_d2.as_rw_storage_bind()) // TODO doesn't have to be writable
            .add(&buffers.voxels.as_ro_bind())
            .add(&buffers.pending_voxels.as_rw_bind())
            .build(device);

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("strolle_voxel_shading_pipeline_layout"),
                bind_group_layouts: &[bg0.as_ref(), bg1.as_ref()],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::COMPUTE,
                    range: Range {
                        start: 0,
                        end: mem::size_of::<gpu::VoxelShadingPassParams>()
                            as u32,
                    },
                }],
            });

        let pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("strolle_voxel_shading_pipeline"),
                layout: Some(&pipeline_layout),
                module: &engine.shaders.voxel_shading,
                entry_point: "main",
            });

        Self { bg0, bg1, pipeline }
    }

    pub fn run<P>(
        &self,
        camera: &CameraController<P>,
        encoder: &mut wgpu::CommandEncoder,
        seed: u32,
    ) where
        P: Params,
    {
        let mut pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("strolle_voxel_shading_pass"),
            });

        let params = gpu::VoxelShadingPassParams {
            frame: camera.frame,
            seed,
        };

        // This pass uses 8x8 warps and the pending-voxels texture has 1/4th of
        // the camera's resolution:
        let size = camera.camera.viewport.size / 8 / 2;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bg0.as_ref(), &[]);
        pass.set_bind_group(1, self.bg1.as_ref(), &[]);
        pass.set_push_constants(0, bytemuck::bytes_of(&params));
        pass.dispatch_workgroups(size.x, size.y, 1);
    }
}
