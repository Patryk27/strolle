use log::debug;

use crate::{Camera, CameraBuffers, Engine, Params};

macro_rules! passes {
    ([ $( $name:ident => $class:ident, )* ]) => {
        $( mod $name; )*
        $( pub use self::$name::*; )*

        #[derive(Debug)]
        pub struct CameraPasses {
            $( pub $name: $class, )*
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

                Self {
                    $( $name: $class::new(engine, device, config, buffers), )*
                }
            }
        }
    };
}

passes!([
    atmosphere => AtmospherePass,
    bvh_heatmap => BvhHeatmapPass,
    di_resolving => DiResolvingPass,
    di_sampling => DiSamplingPass,
    di_spatial_resampling => DiSpatialResamplingPass,
    di_temporal_resampling => DiTemporalResamplingPass,
    frame_composition => FrameCompositionPass,
    frame_denoising => FrameDenoisingPass,
    frame_reprojection => FrameReprojectionPass,
    gi_preview_resampling => GiPreviewResamplingPass,
    gi_reprojection => GiReprojectionPass,
    gi_resolving => GiResolvingPass,
    gi_sampling => GiSamplingPass,
    gi_spatial_resampling => GiSpatialResamplingPass,
    gi_temporal_resampling => GiTemporalResamplingPass,
    prim_raster => PrimRasterPass,
    ref_shading => RefShadingPass,
    ref_tracing => RefTracingPass,
]);
