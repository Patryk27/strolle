use crate::{Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct DirectResolvingPass {
    pass: CameraComputePass,
}

impl DirectResolvingPass {
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass = CameraComputePass::builder("direct_resolving")
            .bind([
                &engine.lights.bind_readable(),
                &engine.world.bind_readable(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.atmosphere_transmittance_lut.bind_sampled(),
                &buffers.atmosphere_sky_lut.bind_sampled(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.direct_next_reservoirs.bind_readable(),
                &buffers.direct_prev_reservoirs.bind_writable(),
                &buffers.direct_samples.bind_writable(),
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
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size, camera.pass_params());
    }
}
