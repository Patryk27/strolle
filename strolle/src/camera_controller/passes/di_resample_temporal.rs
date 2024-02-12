use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DiResampleTemporalPass {
    pass: CameraComputePass,
}

impl DiResampleTemporalPass {
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
        let pass = CameraComputePass::builder("di_resample_temporal")
            .bind([&engine.lights.bind_readable()])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.rt_hits.bind_readable(),
                &buffers.di_prev_reservoirs.bind_readable(),
                &buffers.di_curr_reservoirs.bind_writable(),
            ])
            .build(device, &engine.shaders.di_resample_temporal);

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
