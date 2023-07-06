use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectDenoisingPass {
    pass: CameraComputePass<()>,
}

impl IndirectDenoisingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_denoising")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.surface_map.prev().bind_readable(),
                &buffers.raw_indirect_colors.bind_readable(),
                &buffers.indirect_colors.curr().bind_writable(),
                &buffers.indirect_colors.prev().bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_denoising);

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
