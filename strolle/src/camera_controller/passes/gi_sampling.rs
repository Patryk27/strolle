use glam::uvec2;

use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};
use crate::utils::ToGpu;

#[derive(Debug)]
pub struct GiSamplingPass {
    pass_a: CameraComputePass,
    pass_b: CameraComputePass,
}

impl GiSamplingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass_a = CameraComputePass::builder("gi_sampling_a")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
            ])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.gi_d0.bind_writable(),
                &buffers.gi_d1.bind_writable(),
                &buffers.gi_d2.bind_writable(),
                &buffers.gi_reservoirs[2].bind_readable(),
            ])
            .build(device, &engine.shaders.gi_sampling_a);

        let pass_b = CameraComputePass::builder("gi_sampling_b")
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
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.gi_d0.bind_readable(),
                &buffers.gi_d1.bind_readable(),
                &buffers.gi_d2.bind_readable(),
                &buffers.gi_reservoirs[2].bind_readable(),
                &buffers.gi_reservoirs[1].bind_writable(),
            ])
            .build(device, &engine.shaders.gi_sampling_b);

        Self { pass_a, pass_b }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // These passes use 8x8 warps and 2x1 checkerboard:
        let size = (camera.camera.viewport.size + 7) / 8 / uvec2(2, 1);

        self.pass_a.run(camera, encoder, size.to_gpu(), camera.pass_params());
        self.pass_b.run(camera, encoder, size.to_gpu(), camera.pass_params());
    }
}
