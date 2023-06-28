use crate::{
    CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct ReprojectionPass {
    pass: CameraComputePass,
}

impl ReprojectionPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("reprojection")
            .bind([
                &buffers.past_camera.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.surface_map.past().bind_readable(),
                &buffers.velocity_map.bind_readable(),
                &buffers.reprojection_map.bind_writable(),
            ])
            .build(device, &engine.shaders.reprojection);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = camera.camera.viewport.size / 8;

        self.pass.run(camera, encoder, size, &());
    }
}
