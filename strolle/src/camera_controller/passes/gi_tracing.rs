use glam::uvec2;
use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiTracingPass {
    pass: CameraComputePass<gpu::GiPassParams>,
}

impl GiTracingPass {
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
        let pass = CameraComputePass::builder("gi_tracing")
            .bind([
                &engine.noise.bind_blue_noise_sobol(),
                &engine.noise.bind_blue_noise_scrambling_tile(),
                &engine.noise.bind_blue_noise_ranking_tile(),
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                &buffers.gi_rays.bind_writable(),
                &buffers.gi_gbuffer_d0.bind_writable(),
                &buffers.gi_gbuffer_d1.bind_writable(),
            ])
            .build(device, &engine.shaders.gi_tracing);

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
