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
    direct_denoising => DirectDenoisingPass,
    direct_raster => DirectRasterPass,
    direct_resolving => DirectResolvingPass,
    direct_shading => DirectShadingPass,
    direct_spatial_resampling => DirectSpatialResamplingPass,
    direct_temporal_resampling => DirectTemporalResamplingPass,
    frame_composition => FrameCompositionPass,
    frame_reprojection => FrameReprojectionPass,
    indirect_diffuse_denoising => IndirectDiffuseDenoisingPass,
    indirect_diffuse_resolving => IndirectDiffuseResolvingPass,
    indirect_diffuse_spatial_resampling => IndirectDiffuseSpatialResamplingPass,
    indirect_diffuse_temporal_resampling => IndirectDiffuseTemporalResamplingPass,
    indirect_shading => IndirectShadingPass,
    indirect_specular_denoising => IndirectSpecularDenoisingPass,
    indirect_specular_resampling => IndirectSpecularResamplingPass,
    indirect_specular_resolving => IndirectSpecularResolvingPass,
    indirect_tracing => IndirectTracingPass,
    reference_shading => ReferenceShadingPass,
    reference_tracing => ReferenceTracingPass,
]);
