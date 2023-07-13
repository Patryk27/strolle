mod atmosphere;
mod bvh_heatmap;
mod direct_denoising;
mod direct_initial_shading;
mod direct_raster;
mod direct_resolving;
mod direct_secondary_tracing;
mod direct_spatial_resampling;
mod direct_temporal_resampling;
mod frame_composition;
mod frame_reprojection;
mod indirect_denoising;
mod indirect_initial_shading;
mod indirect_initial_tracing;
mod indirect_resolving;
mod indirect_spatial_resampling;
mod indirect_temporal_resampling;

use log::debug;

pub use self::atmosphere::*;
pub use self::bvh_heatmap::*;
pub use self::direct_denoising::*;
pub use self::direct_initial_shading::*;
pub use self::direct_raster::*;
pub use self::direct_resolving::*;
pub use self::direct_secondary_tracing::*;
pub use self::direct_spatial_resampling::*;
pub use self::direct_temporal_resampling::*;
pub use self::frame_composition::*;
pub use self::frame_reprojection::*;
pub use self::indirect_denoising::*;
pub use self::indirect_initial_shading::*;
pub use self::indirect_initial_tracing::*;
pub use self::indirect_resolving::*;
pub use self::indirect_spatial_resampling::*;
pub use self::indirect_temporal_resampling::*;
use crate::{Camera, CameraBuffers, Engine, Params};

#[derive(Debug)]
pub struct CameraPasses {
    pub atmosphere: AtmospherePass,
    pub bvh_heatmap: BvhHeatmapPass,

    pub direct_denoising: DirectDenoisingPass,
    pub direct_initial_shading: DirectInitialShadingPass,
    pub direct_raster: DirectRasterPass,
    pub direct_resolving: DirectResolvingPass,
    pub direct_secondary_tracing: DirectSecondaryTracingPass,
    pub direct_spatial_resampling: DirectSpatialResamplingPass,
    pub direct_temporal_resampling: DirectTemporalResamplingPass,

    pub indirect_denoising: IndirectDenoisingPass,
    pub indirect_initial_shading: IndirectInitialShadingPass,
    pub indirect_initial_tracing: IndirectInitialTracingPass,
    pub indirect_resolving: IndirectResolvingPass,
    pub indirect_spatial_resampling: IndirectSpatialResamplingPass,
    pub indirect_temporal_resampling: IndirectTemporalResamplingPass,

    pub frame_composition: FrameCompositionPass,
    pub frame_reprojection: FrameReprojectionPass,
}

impl CameraPasses {
    pub fn new<P>(
        engine: &Engine<P>,
        device: &wgpu::Device,
        config: &Camera,
        buffers: &CameraBuffers,
    ) -> Self
    where
        P: Params,
    {
        debug!("Initializing camera passes");

        // TODO use a macro to build them
        Self {
            atmosphere: AtmospherePass::new(engine, device, buffers),
            bvh_heatmap: BvhHeatmapPass::new(engine, device, buffers),

            direct_denoising: DirectDenoisingPass::new(engine, device, buffers),
            direct_initial_shading: DirectInitialShadingPass::new(
                engine, device, buffers,
            ),
            direct_raster: DirectRasterPass::new(
                engine, device, config, buffers,
            ),
            direct_resolving: DirectResolvingPass::new(engine, device, buffers),
            direct_secondary_tracing: DirectSecondaryTracingPass::new(
                engine, device, buffers,
            ),
            direct_spatial_resampling: DirectSpatialResamplingPass::new(
                engine, device, buffers,
            ),
            direct_temporal_resampling: DirectTemporalResamplingPass::new(
                engine, device, buffers,
            ),

            indirect_denoising: IndirectDenoisingPass::new(
                engine, device, buffers,
            ),
            indirect_initial_shading: IndirectInitialShadingPass::new(
                engine, device, buffers,
            ),
            indirect_initial_tracing: IndirectInitialTracingPass::new(
                engine, device, buffers,
            ),
            indirect_resolving: IndirectResolvingPass::new(
                engine, device, buffers,
            ),
            indirect_spatial_resampling: IndirectSpatialResamplingPass::new(
                engine, device, buffers,
            ),
            indirect_temporal_resampling: IndirectTemporalResamplingPass::new(
                engine, device, buffers,
            ),

            frame_composition: FrameCompositionPass::new(
                engine, device, config, buffers,
            ),
            frame_reprojection: FrameReprojectionPass::new(
                engine, device, buffers,
            ),
        }
    }
}
