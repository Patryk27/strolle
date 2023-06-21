use rand::Rng;

use crate::{
    gpu, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectSpatialResamplingPass {
    pass: CameraComputePass<gpu::DirectSpatialResamplingPassParams>,
}

impl DirectSpatialResamplingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("direct_spatial_resampling")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.direct_temporal_reservoirs.curr().bind_readable(),
                &buffers.direct_spatial_reservoirs.curr().bind_writable(),
                &buffers.direct_spatial_reservoirs.past().bind_readable(),
            ])
            .build(device, &engine.shaders.direct_spatial_resampling);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        let params = gpu::DirectSpatialResamplingPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}