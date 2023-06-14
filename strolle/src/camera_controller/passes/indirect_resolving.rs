use rand::Rng;
use strolle_gpu as gpu;

use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectResolvingPass {
    pass: CameraComputePass<gpu::IndirectResolvingPassParams>,
}

impl IndirectResolvingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_resolving")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.geometry_map.curr().bind_readable(),
                &buffers.raw_indirect_colors.bind_writable(),
                &buffers.indirect_spatial_reservoirs.curr().bind_readable(),
            ])
            .build(device, &engine.shaders.indirect_resolving);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        let params = gpu::IndirectResolvingPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}
