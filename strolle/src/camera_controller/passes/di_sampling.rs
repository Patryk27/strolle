use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};
use crate::utils::ToGpu;

#[derive(Debug)]
pub struct DiSamplingPass {
    pass: CameraComputePass,
}

impl DiSamplingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("di_sampling")
            .bind([
                &engine.noise.bind_blue_noise(),
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.lights.bind_readable(),
                &engine.images.bind_atlas(),
                &engine.world.bind_readable(),
            ])
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.di_reservoirs[1].bind_writable(),
            ])
            .build(device, &engine.shaders.di_sampling);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size.to_gpu(), camera.pass_params());
    }
}
