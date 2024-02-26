use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiPreviewResamplingPass {
    passes: [CameraComputePass<gpu::GiPreviewResamplingPass>; 2],
}

impl GiPreviewResamplingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass_1 = CameraComputePass::builder("gi_preview_resampling_1")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.prim_surface_map.curr().bind_readable(),
                &buffers.gi_reservoirs[1].bind_readable(),
                &buffers.gi_reservoirs[2].bind_readable(),
                &buffers.gi_reservoirs[3].bind_writable(),
            ])
            .build(device, &engine.shaders.gi_preview_resampling);

        let pass_2 = CameraComputePass::builder("gi_preview_resampling_2")
            .bind([
                &buffers.curr_camera.bind_readable(),
                &buffers.prim_gbuffer_d0.curr().bind_readable(),
                &buffers.prim_gbuffer_d1.curr().bind_readable(),
                &buffers.prim_surface_map.curr().bind_readable(),
                &buffers.gi_reservoirs[1].bind_readable(),
                &buffers.gi_reservoirs[3].bind_readable(),
                &buffers.gi_reservoirs[0].bind_writable(),
            ])
            .build(device, &engine.shaders.gi_preview_resampling);

        Self {
            passes: [pass_1, pass_2],
        }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        source: u32,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;
        let params = camera.pass_params();

        for (nth, pass) in self.passes.iter().enumerate() {
            let source = if nth == 0 { source } else { 1 };

            pass.run(
                camera,
                encoder,
                size,
                gpu::GiPreviewResamplingPass {
                    seed: params.seed,
                    frame: params.frame,
                    nth: nth as u32,
                    source,
                },
            );
        }
    }
}
