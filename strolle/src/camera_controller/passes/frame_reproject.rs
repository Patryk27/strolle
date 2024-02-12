use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct FrameReprojectPass {
    pass: CameraComputePass<()>,
}

impl FrameReprojectPass {
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
        let pass = CameraComputePass::builder("frame_reproject")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prev_camera.bind_readable(),
                &buffers.prim_surface_map.curr().bind_readable(),
                &buffers.prim_surface_map.prev().bind_readable(),
                &buffers.velocity_map.bind_readable(),
                &buffers.reprojection_map.bind_writable(),
            ])
            .build(device, &engine.shaders.frame_reproject);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size, ());
    }
}
