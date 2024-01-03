use rand::Rng;

use crate::{gpu, Camera, CameraBuffers, CameraComputePass, CameraController};

#[derive(Debug)]
pub struct IndirectShadingPass {
    pass: CameraComputePass<gpu::IndirectPassParams>,
}

impl IndirectShadingPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine: &Engine,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        let pass = CameraComputePass::builder("indirect_shading")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.lights.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
                &engine.world.bind_readable(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.atmosphere_transmittance_lut.bind_sampled(),
                &buffers.atmosphere_sky_lut.bind_sampled(),
                &buffers.direct_gbuffer_d0.bind_readable(),
                &buffers.direct_gbuffer_d1.bind_readable(),
                &buffers.indirect_rays.bind_readable(),
                &buffers.indirect_gbuffer_d0.bind_readable(),
                &buffers.indirect_gbuffer_d1.bind_readable(),
                &buffers.indirect_samples.bind_writable(),
            ])
            .build(device, &engine.shaders.indirect_shading);

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
