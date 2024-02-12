use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DiResolvePass {
    pass: CameraComputePass,
}

impl DiResolvePass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("di_resolve")
            .bind([
                &engine.lights.bind_readable(),
                &engine.world.bind_readable(),
            ])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.atmosphere_transmittance_lut.bind_sampled(),
                &buffers.atmosphere_sky_lut.bind_sampled(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.di_next_reservoirs.bind_readable(),
                &buffers.di_prev_reservoirs.bind_writable(),
                &buffers.di_diff_samples.bind_writable(),
                &buffers.rt_hits.bind_readable(),
            ])
            .build(device, &engine.shaders.di_resolve);

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
