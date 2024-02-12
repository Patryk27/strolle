use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct GiResampleSpatialPass {
    exact_pass: CameraComputePass<gpu::GiDiffSpatialResamplingPassParams>,
    approx_passes:
        [CameraComputePass<gpu::GiDiffSpatialResamplingPassParams>; 2],
}

impl GiResampleSpatialPass {
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
        let exact_pass =
            CameraComputePass::builder("gi_spatial_resample_exact")
                .bind([
                    &engine.triangles.bind_readable(),
                    &engine.bvh.bind_readable(),
                    &engine.materials.bind_readable(),
                    &engine.images.bind_atlas(),
                ])
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_reservoirs[1].bind_readable(),
                    &buffers.gi_reservoirs[2].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_resample_spatial_exact);

        let approx_pass_1 =
            CameraComputePass::builder("gi_spatial_resample_approx")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_reservoirs[2].bind_readable(),
                    &buffers.gi_reservoirs[3].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_resample_spatial_approx);

        let approx_pass_2 =
            CameraComputePass::builder("gi_spatial_resample_approx")
                .bind([
                    &buffers.curr_camera.bind_readable(),
                    &buffers.prim_gbuffer_d0.bind_readable(),
                    &buffers.prim_gbuffer_d1.bind_readable(),
                    &buffers.prim_surface_map.curr().bind_readable(),
                    &buffers.gi_reservoirs[3].bind_readable(),
                    &buffers.gi_reservoirs[0].bind_writable(),
                ])
                .build(device, &engine.shaders.gi_resample_spatial_approx);

        Self {
            exact_pass,
            approx_passes: [approx_pass_1, approx_pass_2],
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

        self.exact_pass.run(
            camera,
            encoder,
            size,
            gpu::GiDiffSpatialResamplingPassParams {
                seed: params.seed,
                frame: params.frame,
                nth: 1,
            },
        );

        for (nth, pass) in self.approx_passes.iter().enumerate() {
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
