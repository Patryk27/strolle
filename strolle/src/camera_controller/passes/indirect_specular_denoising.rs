use rand::Rng;
use strolle_gpu as gpu;

use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectSpecularDenoisingPass {
    pass: CameraComputePass<gpu::PassParams>,
}

impl IndirectSpecularDenoisingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_specular_denoising")
            .bind([
                &buffers.indirect_specular_samples.bind_readable(),
                &buffers.indirect_specular_colors.curr().bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_specular_denoising);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        let params = gpu::PassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}
