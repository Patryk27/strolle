use crate::{gpu, Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct IndirectDiffuseSpatialResamplingPass {
    pass_a: CameraComputePass<gpu::IndirectDiffuseSpatialResamplingPassParams>,
    pass_b: CameraComputePass<gpu::IndirectDiffuseSpatialResamplingPassParams>,
}

impl IndirectDiffuseSpatialResamplingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass_a =
            CameraComputePass::builder("indirect_diffuse_spatial_resampling_a")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.direct_gbuffer_d0.bind_readable(),
                    &buffers.direct_gbuffer_d1.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                    &buffers
                        .indirect_diffuse_temporal_reservoirs
                        .curr()
                        .bind_readable(),
                    &buffers
                        .indirect_diffuse_spatial_reservoirs_a
                        .bind_writable(),
                ])
                .build(
                    device,
                    &engine.shaders.indirect_diffuse_spatial_resampling,
                );

        let pass_b =
            CameraComputePass::builder("indirect_diffuse_spatial_resampling_b")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.direct_gbuffer_d0.bind_readable(),
                    &buffers.direct_gbuffer_d1.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                    &buffers
                        .indirect_diffuse_spatial_reservoirs_a
                        .bind_readable(),
                    &buffers
                        .indirect_diffuse_spatial_reservoirs_b
                        .bind_writable(),
                ])
                .build(
                    device,
                    &engine.shaders.indirect_diffuse_spatial_resampling,
                );

        Self { pass_a, pass_b }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        let params_a = camera.pass_params();
        let params_b = camera.pass_params();

        self.pass_a.run(
            camera,
            encoder,
            size,
            gpu::IndirectDiffuseSpatialResamplingPassParams {
                seed: params_a.seed,
                frame: params_a.frame,
                nth: 1,
            },
        );

        self.pass_b.run(
            camera,
            encoder,
            size,
            gpu::IndirectDiffuseSpatialResamplingPassParams {
                seed: params_b.seed,
                frame: params_b.frame,
                nth: 2,
            },
        );
    }
}
