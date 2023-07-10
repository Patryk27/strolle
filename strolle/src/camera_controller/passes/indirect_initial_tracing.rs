use crate::{
    gpu, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectInitialTracingPass {
    pass: CameraComputePass<gpu::IndirectInitialTracingPassParams>,
}

impl IndirectInitialTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_initial_tracing")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
            ])
            .bind([
                &buffers.direct_primary_hits_d0.bind_readable(),
                &buffers.direct_primary_hits_d1.bind_readable(),
                &buffers.indirect_hits_d0.bind_writable(),
                &buffers.indirect_hits_d1.bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_initial_tracing);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        seed: u32,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        let params = gpu::IndirectInitialTracingPassParams {
            seed,
            frame: camera.frame,
        };

        self.pass.run(camera, encoder, size, &params);
    }
}
