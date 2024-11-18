use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};
use crate::utils::ToGpu;

#[derive(Debug)]
pub struct GiResolvingPass {
    pass: CameraComputePass<gpu::GiResolvingPassParams>,
}

impl GiResolvingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("gi_resolving")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.gi_reservoirs[1].bind_readable(),
                &buffers.gi_reservoirs[2].bind_readable(),
                &buffers.gi_reservoirs[0].bind_writable(),
                &buffers.gi_diff_samples.bind_writable(),
                &buffers.gi_spec_samples.bind_writable(),
            ])
            .build(device, &engine.shaders.gi_resolving);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        source: u32,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(
            camera,
            encoder,
            size.to_gpu(),
            gpu::GiResolvingPassParams {
                frame: camera.frame,
                source,
            },
        );
    }
}
