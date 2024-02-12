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
    di_resample_spatial_s0 => DiResampleSpatialS0Pass,
    di_resample_spatial_s1 => DiResampleSpatialS1Pass,
    di_resample_temporal => DiResampleTemporalPass,
    di_resolve => DiResolvePass,
    di_sample => DiSamplePass,
    frame_compose => FrameComposePass,
    frame_denoise => FrameDenoisePass,
    frame_reproject => FrameReprojectPass,
    gi_prepare => GiPreparePass,
    gi_resample_spatial => GiResampleSpatialPass,
    gi_resample_temporal => GiResampleTemporalPass,
    gi_resolve => GiResolvePass,
    gi_sample => GiSamplePass,
    prim_raster => PrimRasterPass,
    ref_shade => RefShadePass,
    ref_trace => RefTracePass,
    rt_intersect => RtIntersectPass,
]);
