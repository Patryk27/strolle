use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct IndirectDiffuseDenoisingPass {
    pass: CameraComputePass,
}

impl IndirectDiffuseDenoisingPass {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        _: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        let pass = CameraComputePass::builder("indirect_diffuse_denoising")
            .bind([&engine.noise.bind_blue_noise_texture()])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.reprojection_map.bind_readable(),
                &buffers.surface_map.curr().bind_readable(),
                &buffers.surface_map.prev().bind_readable(),
                &buffers.indirect_diffuse_samples.bind_readable(),
                &buffers.indirect_diffuse_colors.curr().bind_writable(),
                &buffers.indirect_diffuse_colors.prev().bind_readable(),
            ])
            .build(device, &engine.shaders.indirect_diffuse_denoising);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size, camera.pass_params());
    }
}
