use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct GiResolvePass {
    pass: CameraComputePass,
}

impl GiResolvePass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("gi_resolve")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.gi_reservoirs[2].bind_readable(),
                &buffers.gi_reservoirs[0].bind_writable(),
                &buffers.gi_diff_samples.bind_writable(),
            ])
            .build(device, &engine.shaders.gi_resolve);

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
