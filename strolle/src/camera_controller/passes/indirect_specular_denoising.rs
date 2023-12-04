use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectSpecularDenoisingPass {
    pass: CameraComputePass,
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
                &buffers.camera.bind_readable(),
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
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size, camera.pass_params());
    }
}
