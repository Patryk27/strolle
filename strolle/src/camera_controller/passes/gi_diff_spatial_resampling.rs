use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiDiffSpatialResamplingPass {
    slow_pass: CameraComputePass<gpu::GiDiffSpatialResamplingPassParams>,
    fast_passes: [CameraComputePass<gpu::GiDiffSpatialResamplingPassParams>; 2],
}

impl GiDiffSpatialResamplingPass {
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
        let slow_pass =
            CameraComputePass::builder("gi_diff_spatial_resampling_slow")
                .bind([
                    &engine.triangles.bind_readable(),
                    &engine.bvh.bind_readable(),
                    &engine.materials.bind_readable(),
                    &engine.images.bind_atlas(),
                ])
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_diff_reservoirs[1].bind_readable(),
                    &buffers.gi_diff_reservoirs[2].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_diff_spatial_resampling_slow);

        let fast_pass_1 =
            CameraComputePass::builder("gi_diff_spatial_resampling_fast")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_diff_reservoirs[2].bind_readable(),
                    &buffers.gi_diff_reservoirs[3].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_diff_spatial_resampling_fast);

        let fast_pass_2 =
            CameraComputePass::builder("gi_diff_spatial_resampling_fast")
                .bind([
                    &buffers.camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_diff_reservoirs[3].bind_readable(),
                    &buffers.gi_diff_reservoirs[0].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_diff_spatial_resampling_fast);

        Self {
            slow_pass,
            fast_passes: [fast_pass_1, fast_pass_2],
        }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;
        let params = camera.pass_params();

        self.slow_pass.run(
            camera,
            encoder,
            size,
            gpu::GiDiffSpatialResamplingPassParams {
                seed: params.seed,
                frame: params.frame,
                nth: 1,
            },
        );

        for (nth, pass) in self.fast_passes.iter().enumerate() {
            pass.run(
                camera,
                encoder,
                size,
                gpu::GiDiffSpatialResamplingPassParams {
                    seed: params.seed,
                    frame: params.frame,
                    nth: nth as u32,
                },
            );
        }
    }
}
