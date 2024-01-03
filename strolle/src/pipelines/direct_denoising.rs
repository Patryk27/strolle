use crate::{gpu, Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct DirectDenoisingPass {
    reproject_pass: CameraComputePass,
    estimate_variance_pass: CameraComputePass,
    wavelet_pass_1: CameraComputePass<gpu::DirectDenoisingPassParams>,
    wavelet_pass_2: CameraComputePass<gpu::DirectDenoisingPassParams>,
    wavelet_pass_3: CameraComputePass<gpu::DirectDenoisingPassParams>,
    wavelet_pass_4: CameraComputePass<gpu::DirectDenoisingPassParams>,
}

impl DirectDenoisingPass {
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let reproject_pass =
            CameraComputePass::builder("direct_denoising_reproject")
                .with_entry_point("reproject")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                    &buffers.direct_samples.bind_readable(),
                    &buffers.direct_colors_a.bind_readable(),
                    &buffers.direct_colors_b.bind_writable(),
                    &buffers.direct_moments.prev().bind_readable(),
                    &buffers.direct_moments.curr().bind_writable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        let estimate_variance_pass =
            CameraComputePass::builder("direct_denoising_estimate_variance")
                .with_entry_point("estimate_variance")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.direct_colors_b.bind_writable(),
                    &buffers.direct_moments.curr().bind_readable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        let wavelet_pass_1 =
            CameraComputePass::builder("direct_denoising_wavelet")
                .with_entry_point("wavelet")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.direct_colors_b.bind_readable(),
                    &buffers.direct_colors_a.bind_writable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        let wavelet_pass_2 =
            CameraComputePass::builder("direct_denoising_wavelet")
                .with_entry_point("wavelet")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.direct_colors_a.bind_readable(),
                    &buffers.direct_colors_b.bind_writable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        let wavelet_pass_3 =
            CameraComputePass::builder("direct_denoising_wavelet")
                .with_entry_point("wavelet")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.direct_colors_b.bind_readable(),
                    &buffers.direct_colors_c.bind_writable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        let wavelet_pass_4 =
            CameraComputePass::builder("direct_denoising_wavelet")
                .with_entry_point("wavelet")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.surface_map.curr().bind_readable(),
                    &buffers.direct_colors_c.bind_readable(),
                    &buffers.direct_colors_b.bind_writable(),
                ])
                .build(device, &engine.shaders.direct_denoising);

        Self {
            reproject_pass,
            estimate_variance_pass,
            wavelet_pass_1,
            wavelet_pass_2,
            wavelet_pass_3,
            wavelet_pass_4,
        }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.reproject_pass
            .run(camera, encoder, size, camera.pass_params());

        self.estimate_variance_pass.run(
            camera,
            encoder,
            size,
            camera.pass_params(),
        );

        self.wavelet_pass_1.run(
            camera,
            encoder,
            size,
            gpu::DirectDenoisingPassParams { stride: 1 },
        );

        self.wavelet_pass_2.run(
            camera,
            encoder,
            size,
            gpu::DirectDenoisingPassParams { stride: 2 },
        );

        self.wavelet_pass_3.run(
            camera,
            encoder,
            size,
            gpu::DirectDenoisingPassParams { stride: 4 },
        );

        self.wavelet_pass_4.run(
            camera,
            encoder,
            size,
            gpu::DirectDenoisingPassParams { stride: 8 },
        );
    }
}
