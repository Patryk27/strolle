mod denoising;
mod drawing;
mod raster;
mod ray_shading;
mod voxel_painting;
mod voxel_shading;
mod voxel_tracing;

pub use self::denoising::*;
pub use self::drawing::*;
pub use self::raster::*;
pub use self::ray_shading::*;
pub use self::voxel_painting::*;
pub use self::voxel_shading::*;
pub use self::voxel_tracing::*;
use super::buffers::CameraBuffers;
use crate::{Camera, Engine, EventHandler, EventHandlerContext, Params};

#[derive(Debug)]
pub struct CameraPasses<P>
where
    P: Params,
{
    pub denoising: DenoisingPass,
    pub drawing: DrawingPass,
    pub raster: RasterPass<P>,
    pub ray_shading: RayShadingPass,
    pub voxel_painting: VoxelPaintingPass,
    pub voxel_shading: VoxelShadingPass,
    pub voxel_tracing: VoxelTracingPass,
}

impl<P> CameraPasses<P>
where
    P: Params,
{
    pub fn new(
        engine: &Engine<P>,
        device: &wgpu::Device,
        config: &Camera,
        buffers: &CameraBuffers,
    ) -> Self {
        log::debug!("Initializing camera passes");

        Self {
            denoising: DenoisingPass::new(engine, device, buffers),
            drawing: DrawingPass::new(engine, device, config, buffers),
            raster: RasterPass::new(engine, device, config, buffers),
            ray_shading: RayShadingPass::new(engine, device, buffers),
            voxel_painting: VoxelPaintingPass::new(engine, device, buffers),
            voxel_shading: VoxelShadingPass::new(engine, device, buffers),
            voxel_tracing: VoxelTracingPass::new(engine, device, buffers),
        }
    }
}

impl<P> EventHandler<P> for CameraPasses<P>
where
    P: Params,
{
    fn handle(&mut self, ctxt: EventHandlerContext<P>) {
        self.raster.handle(ctxt);
    }
}
