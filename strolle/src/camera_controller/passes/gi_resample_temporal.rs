use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct GiResampleTemporalPass {
    pass: CameraComputePass,
}

impl GiResampleTemporalPass {
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
        let pass = CameraComputePass::builder("gi_resample_temporal")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_surface_map.curr().bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.gi_samples.bind_readable(),
                &buffers.gi_reservoirs[0].bind_readable(),
                &buffers.gi_reservoirs[1].bind_writable(),
            ])
            .build(device, &engine.shaders.gi_resample_temporal);

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
