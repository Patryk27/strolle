use glam::uvec2;
use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiSamplePass {
    pass: CameraComputePass<gpu::GiPassParams>,
}

impl GiSamplePass {
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
        let pass = CameraComputePass::builder("gi_sample")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.lights.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
                &engine.world.bind_readable(),
            ])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.atmosphere_transmittance_lut.bind_sampled(),
                &buffers.atmosphere_sky_lut.bind_sampled(),
                &buffers.prim_gbuffer_d0.bind_readable(),
                &buffers.prim_gbuffer_d1.bind_readable(),
                // &buffers.gi_rays.bind_readable(),
                // &buffers.gi_gbuffer_d0.bind_readable(),
                // &buffers.gi_gbuffer_d1.bind_readable(),
                &buffers.gi_samples.bind_writable(),
            ])
            .build(device, &engine.shaders.gi_sample);

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
