use crate::buffers::Texture;
use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct FrameDenoisePass {
    reproject_passes: [CameraComputePass; 2],
    estimate_variance_pass: CameraComputePass,
    wavelet_passes:
        [CameraComputePass<gpu::FrameDenoisingWaveletPassParams>; 5],
}

impl FrameDenoisePass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let reproject_di_pass =
            CameraComputePass::builder("frame_denoise_reproject_di")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                ])
                .bind([
                    &buffers.di_diff_prev_colors.bind_readable(),
                    &buffers.di_diff_moments.prev().bind_readable(),
                    &buffers.di_diff_samples.bind_readable(),
                    &buffers.di_diff_curr_colors.bind_writable(),
                    &buffers.di_diff_moments.curr().bind_writable(),
                ])
                .build(device, &engine.shaders.frame_denoise_reproject);

        let reproject_gi_pass =
            CameraComputePass::builder("frame_denoise_reproject_gi")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.reprojection_map.bind_readable(),
                ])
                .bind([
                    &buffers.gi_diff_prev_colors.bind_readable(),
                    &buffers.gi_diff_moments.prev().bind_readable(),
                    &buffers.gi_diff_samples.bind_readable(),
                    &buffers.gi_diff_curr_colors.bind_writable(),
                    &buffers.gi_diff_moments.curr().bind_writable(),
                ])
                .build(device, &engine.shaders.frame_denoise_reproject);

        let estimate_variance_pass =
            CameraComputePass::builder("frame_denoise_estimate_variance")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                ])
                .bind([
                    &buffers.di_diff_curr_colors.bind_readable(),
                    &buffers.di_diff_moments.curr().bind_readable(),
                    &buffers.di_diff_stash.bind_writable(),
                ])
                .bind([
                    &buffers.gi_diff_curr_colors.bind_readable(),
                    &buffers.gi_diff_moments.curr().bind_readable(),
                    &buffers.gi_diff_stash.bind_writable(),
                ])
                .build(
                    device,
                    &engine.shaders.frame_denoise_estimate_variance,
                );

        struct WaveletPass<'a> {
            di: (&'a Texture, &'a Texture),
            gi: (&'a Texture, &'a Texture),
        }

        let wavelet_passes = {
            let b = buffers;

            [
                WaveletPass {
                    di: (&b.di_diff_stash, &b.di_diff_prev_colors),
                    gi: (&b.gi_diff_stash, &b.gi_diff_prev_colors),
                },
                WaveletPass {
                    di: (&b.di_diff_prev_colors, &b.di_diff_stash),
                    gi: (&b.gi_diff_prev_colors, &b.gi_diff_stash),
                },
                WaveletPass {
                    di: (&b.di_diff_stash, &b.di_diff_curr_colors),
                    gi: (&b.gi_diff_stash, &b.gi_diff_curr_colors),
                },
                WaveletPass {
                    di: (&b.di_diff_curr_colors, &b.di_diff_stash),
                    gi: (&b.gi_diff_curr_colors, &b.gi_diff_stash),
                },
                WaveletPass {
                    di: (&b.di_diff_stash, &b.di_diff_curr_colors),
                    gi: (&b.gi_diff_stash, &b.gi_diff_curr_colors),
                },
            ]
        };

        let wavelet_passes = {
            let mut n = 0;

            wavelet_passes.map(|wavelet| {
                let label = format!("frame_denoise_wavelet_{}", n);

                n += 1;

                CameraComputePass::builder(label)
                    .bind([
                        &engine.noise.bind_blue_noise(),
                        &buffers.curr_camera.bind_readable(),
                        &buffers.prim_surface_map.curr().bind_readable(),
                    ])
                    .bind([
                        &wavelet.di.0.bind_readable(),
                        &wavelet.di.1.bind_writable(),
                    ])
                    .bind([
                        &wavelet.gi.0.bind_readable(),
                        &wavelet.gi.1.bind_writable(),
                    ])
                    .build(device, &engine.shaders.frame_denoise_wavelet)
            })
        };

        Self {
            reproject_passes: [reproject_di_pass, reproject_gi_pass],
            estimate_variance_pass,
            wavelet_passes,
        }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.reproject_passes[0].run(
            camera,
            encoder,
            size,
            camera.pass_params(),
        );

        self.reproject_passes[1].run(
            camera,
            encoder,
            size,
            camera.pass_params(),
        );

        self.estimate_variance_pass.run(
            camera,
            encoder,
            size,
            camera.pass_params(),
        );

        for (nth, pass) in self.wavelet_passes.iter().enumerate() {
            let nth = nth as u32;

            pass.run(
                camera,
                encoder,
                size,
                gpu::FrameDenoisingWaveletPassParams {
                    frame: camera.frame,
                    stride: 2u32.pow(nth) + nth,
                    strength: (1 + nth) as f32,
                },
            );
        }
    }
}
