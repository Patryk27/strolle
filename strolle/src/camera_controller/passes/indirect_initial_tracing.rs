use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct IndirectInitialTracingPass {
    pass: CameraComputePass<gpu::PassParams>,
}

impl IndirectInitialTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_initial_tracing")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas_sampled(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.direct_hits.bind_readable(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.indirect_rays.bind_writable(),
                &buffers.indirect_gbuffer_d0.bind_writable(),
                &buffers.indirect_gbuffer_d1.bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_initial_tracing);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        self.pass.run(camera, encoder, size, &camera.pass_params());
    }
}
