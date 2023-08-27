use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct IndirectSpecularResamplingPass {
    pass: CameraComputePass<gpu::PassParams>,
}

impl IndirectSpecularResamplingPass {
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
        let pass = CameraComputePass::builder("indirect_specular_resampling")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.prev_camera.bind_readable(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.indirect_samples.bind_readable(),
                &buffers.indirect_specular_reservoirs.curr().bind_writable(),
                &buffers.indirect_specular_reservoirs.prev().bind_readable(),
            ])
            .build(device, &engine.shaders.indirect_specular_resampling);

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
