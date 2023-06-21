use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectResolvingPass {
    pass: CameraComputePass<()>,
}

impl DirectResolvingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("direct_resolving")
            .bind([
                &engine.lights.bind_readable(),
                &engine.materials.bind_readable(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.direct_hits_d0.bind_readable(),
                &buffers.direct_hits_d1.bind_readable(),
                &buffers.direct_hits_d2.bind_readable(),
                &buffers.direct_initial_samples.bind_readable(),
                &buffers.raw_direct_colors.bind_writable(),
                &buffers.direct_spatial_reservoirs.curr().bind_readable(),
            ])
            .build(device, &engine.shaders.direct_resolving);

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
