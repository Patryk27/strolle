use rand::Rng;

use crate::{gpu, Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct IndirectTracingPass {
    pass: CameraComputePass<gpu::IndirectPassParams>,
}

impl IndirectTracingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass = CameraComputePass::builder("indirect_tracing")
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
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.indirect_rays.bind_writable(),
                &buffers.indirect_gbuffer_d0.bind_writable(),
                &buffers.indirect_gbuffer_d1.bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_tracing);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        mode: u32,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        let params = gpu::IndirectPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
            mode,
        };

        self.pass.run(camera, encoder, size, params);
    }
}
