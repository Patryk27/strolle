use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DirectSpatialResamplingPass {
    pass: CameraComputePass,
}

impl DirectSpatialResamplingPass {
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
        let pass = CameraComputePass::builder("direct_spatial_resampling")
            .bind([
                &engine.noise.bind_blue_noise_texture(),
                &engine.lights.bind_readable(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.direct_curr_reservoirs.bind_readable(),
                &buffers.direct_next_reservoirs.bind_writable(),
            ])
            .build(device, &engine.shaders.direct_spatial_resampling);

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
