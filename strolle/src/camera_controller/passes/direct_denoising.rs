use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectDenoisingPass {
    pass: CameraComputePass<()>,
}

impl DirectDenoisingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("direct_denoising")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.raw_direct_colors.bind_readable(),
                &buffers.direct_colors.curr().bind_writable(),
                &buffers.direct_colors.prev().bind_readable(),
            ])
            .build(device, &engine.shaders.direct_denoising);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        self.pass.run(camera, encoder, size, &());
    }
}
