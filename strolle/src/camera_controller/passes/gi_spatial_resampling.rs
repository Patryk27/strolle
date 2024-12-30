use glam::uvec2;

use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};
use crate::utils::ToGpu;

#[derive(Debug)]
pub struct GiSpatialResamplingPass {
    pick_pass: CameraComputePass,
    trace_pass: CameraComputePass,
    sample_pass: CameraComputePass,
}

impl GiSpatialResamplingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pick_pass =
            CameraComputePass::builder("gi_spatial_resampling_pick")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.curr().bind_readable(),
                    &buffers.prim_gbuffer_d1.curr().bind_readable(),
                    &buffers.gi_reservoirs[1].bind_readable(),
                    &buffers.gi_d0.bind_writable(),
                    &buffers.gi_d1.bind_writable(),
                ])
                .build(device, &engine.shaders.gi_spatial_resampling_pick);

        let trace_pass =
            CameraComputePass::builder("gi_spatial_resampling_trace")
                .bind([
                    &engine.triangles.bind_readable(),
                    &engine.bvh.bind_readable(),
                    &engine.materials.bind_readable(),
                    &engine.images.bind_atlas(),
                ])
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.gi_d0.bind_readable(),
                    &buffers.gi_d1.bind_readable(),
                    &buffers.gi_d2.bind_writable(),
                ])
                .build(device, &engine.shaders.gi_spatial_resampling_trace);

        let sample_pass =
            CameraComputePass::builder("gi_spatial_resampling_sample")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.gi_reservoirs[1].bind_readable(),
                    &buffers.gi_reservoirs[2].bind_writable(),
                    &buffers.gi_d2.bind_readable(),
                ])
                .build(device, &engine.shaders.gi_spatial_resampling_sample);

        Self {
            pick_pass,
            trace_pass,
            sample_pass,
        }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.pick_pass.run(
            camera,
            encoder,
            ((camera.camera.viewport.size + 7) / 8 / uvec2(2, 1)).to_gpu(),
            camera.pass_params(),
        );

        self.trace_pass.run(
            camera,
            encoder,
            ((camera.camera.viewport.size + 7) / 8).to_gpu(),
            camera.pass_params(),
        );

        self.sample_pass.run(
            camera,
            encoder,
            ((camera.camera.viewport.size + 7) / 8 / uvec2(2, 1)).to_gpu(),
            camera.pass_params(),
        );
    }
}
