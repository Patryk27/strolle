use rand::Rng;

use crate::{
    gpu, Camera, CameraBuffers, CameraComputePass, CameraController, Engine,
    Params,
};

#[derive(Debug)]
pub struct RefShadingPass {
    pass: CameraComputePass<gpu::RefPassParams>,
}

impl RefShadingPass {
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
        let pass = CameraComputePass::builder("reference_shading")
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
                &buffers.prev_camera.bind_readable(),
                &buffers.atmosphere_transmittance_lut.bind_sampled(),
                &buffers.atmosphere_sky_lut.bind_sampled(),
                &buffers.ref_rays.bind_writable(),
                &buffers.ref_hits.bind_readable(),
                &buffers.ref_colors.bind_writable(),
            ])
            .build(device, &engine.shaders.ref_shading);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
        depth: u8,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        let params = gpu::RefPassParams {
            seed: rand::thread_rng().gen(),
            frame: camera.frame,
            depth: depth as u32,
        };

        self.pass.run(camera, encoder, size, params);
    }
}
