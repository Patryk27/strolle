use glam::uvec2;
use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiPreparePass {
    pass: CameraComputePass<gpu::GiPassParams>,
}

impl GiPreparePass {
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
        let pass = CameraComputePass::builder("gi_prepare")
            .bind([
                &engine.noise.bind_blue_noise_sobol(),
                &engine.noise.bind_blue_noise_scrambling_tile(),
                &engine.noise.bind_blue_noise_ranking_tile(),
            ])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.rt_rays.bind_writable(),
            ])
            .build(device, &engine.shaders.gi_prepare);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        mode: u32,
    ) {
        // This pass uses 8x8 warps and 2x1 checkerboard:
        let size = (camera.camera.viewport.size + 7) / 8 / uvec2(2, 1);

        let params = gpu::GiPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
            mode,
        };

        self.pass.run(camera, encoder, size, params);
    }
}
