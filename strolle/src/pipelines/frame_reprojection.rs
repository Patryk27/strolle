use crate::{Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct FrameReprojectionPass {
    pass: CameraComputePass<()>,
}

impl FrameReprojectionPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass = CameraComputePass::builder("frame_reprojection")
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.prev_camera.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.surface_map.prev().bind_readable(),
                &buffers.velocity_map.bind_readable(),
                &buffers.reprojection_map.bind_writable(),
            ])
            .build(device, &engine.shaders.frame_reprojection);

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
