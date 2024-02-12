use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct DiResampleSpatialS1Pass {
    pass: CameraComputePass,
}

impl DiResampleSpatialS1Pass {
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
        let pass = CameraComputePass::builder("di_resample_spatial_s1")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.di_next_reservoirs.bind_writable(),
                &buffers.rt_rays.bind_writable(),
                &buffers.rt_hits.bind_readable(),
            ])
            .build(device, &engine.shaders.di_resample_spatial_s1);

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
