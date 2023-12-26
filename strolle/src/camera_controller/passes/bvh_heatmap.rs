use crate::{
    Camera, CameraBuffers, CameraComputePass, CameraController, Engine, Params,
};

#[derive(Debug)]
pub struct BvhHeatmapPass {
    pass: CameraComputePass<()>,
}

impl BvhHeatmapPass {
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
        let pass = CameraComputePass::builder("bvh_heatmap")
            .bind([
                &engine.triangles.bind_readable(),
                &engine.bvh.bind_readable(),
                &engine.materials.bind_readable(),
                &engine.images.bind_atlas(),
            ])
            .bind([
                &buffers.camera.bind_readable(),
                &buffers.direct_colors_b.bind_writable(),
            ])
            .build(device, &engine.shaders.bvh_heatmap);

        Self { pass }
    }

    pub fn run(
        &self,
        camera: &CameraController,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // This pass uses 8x8 warps:
        let size = (camera.camera.viewport.size + 7) / 8;

        self.pass.run(camera, encoder, size, ());
    }
}
