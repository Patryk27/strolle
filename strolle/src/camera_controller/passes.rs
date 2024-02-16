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
    di_shading => DiShadingPass,
    di_spatial_resampling => DiSpatialResamplingPass,
    di_temporal_resampling => DiTemporalResamplingPass,
    frame_composition => FrameCompositionPass,
    frame_denoising => FrameDenoisingPass,
    frame_reprojection => FrameReprojectionPass,
    gi_diff_resolving => GiDiffResolvingPass,
    gi_diff_spatial_resampling => GiDiffSpatialResamplingPass,
    gi_diff_temporal_resampling => GiDiffTemporalResamplingPass,
    gi_shading => GiShadingPass,
    gi_spec_resampling => GiSpecResamplingPass,
    gi_spec_resolving => GiSpecResolvingPass,
    gi_tracing => GiTracingPass,
    prim_raster => PrimRasterPass,
    ref_shading => RefShadingPass,
    ref_tracing => RefTracingPass,
]);
