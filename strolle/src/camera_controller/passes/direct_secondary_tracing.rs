use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectSecondaryTracingPass {
    pass: CameraComputePass<()>,
}

impl DirectSecondaryTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("direct_secondary_tracing")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_sampled(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.direct_primary_hits_d0.bind_readable(),
                &buffers.direct_primary_hits_d1.bind_readable(),
                &buffers.direct_primary_hits_d2.bind_readable(),
                &buffers.direct_secondary_rays.bind_writable(),
                &buffers.direct_secondary_hits_d0.bind_writable(),
                &buffers.direct_secondary_hits_d1.bind_writable(),
                &buffers.direct_secondary_hits_d2.bind_writable(),
            ])
            .build(device, &engine.shaders.direct_secondary_tracing);

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
