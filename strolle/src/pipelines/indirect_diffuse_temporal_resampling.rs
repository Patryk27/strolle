use crate::{Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct IndirectDiffuseTemporalResamplingPass {
    pass: CameraComputePass,
}

impl IndirectDiffuseTemporalResamplingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass =
            CameraComputePass::builder("indirect_diffuse_temporal_resampling")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.surface_map.prev().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                    &buffers.indirect_samples.bind_readable(),
                    &buffers
                        .indirect_diffuse_temporal_reservoirs
                        .curr()
                        .bind_writable(),
                    &buffers
                        .indirect_diffuse_temporal_reservoirs
                        .prev()
                        .bind_readable(),
                ])
                .build(
                    device,
                    &engine.shaders.indirect_diffuse_temporal_resampling,
                );

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
