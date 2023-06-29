use rand::Rng;

use crate::{
    gpu, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectSpatialResamplingPass {
    pass: CameraComputePass<gpu::IndirectSpatialResamplingPassParams>,
}

impl IndirectSpatialResamplingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_spatial_resampling")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.direct_hits_d0.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.surface_map.prev().bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.indirect_temporal_reservoirs.curr().bind_readable(),
                &buffers.indirect_spatial_reservoirs.curr().bind_writable(),
                &buffers.indirect_spatial_reservoirs.prev().bind_readable(),
            ])
            .build(device, &engine.shaders.indirect_spatial_resampling);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses half-scaled viewport and 8x8 warps:
        let size = camera.camera.viewport.size / 2 / 8;

        let params = gpu::IndirectSpatialResamplingPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}
