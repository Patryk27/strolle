mod atmosphere;
mod bvh_heatmap;
mod direct_denoising;
mod direct_raster;
mod direct_resolving;
mod direct_seeding;
mod direct_spatial_resampling;
mod direct_temporal_resampling;
mod frame_composition;
mod frame_reprojection;
mod indirect_diffuse_denoising;
mod indirect_diffuse_resolving;
mod indirect_diffuse_spatial_resampling;
mod indirect_diffuse_temporal_resampling;
mod indirect_seeding;
mod indirect_specular_denoising;
mod indirect_specular_resampling;
mod indirect_specular_resolving;
mod indirect_tracing;
mod reference_shading;
mod reference_tracing;

use log::debug;

use crate::{Camera, CameraBuffers, Engine, Params};

macro_rules! passes {
    ([ $( $name:ident => $class:ident, )* ]) => {
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
    direct_seeding => DirectSeedingPass,
    direct_spatial_resampling => DirectSpatialResamplingPass,
    direct_temporal_resampling => DirectTemporalResamplingPass,
    frame_composition => FrameCompositionPass,
    frame_reprojection => FrameReprojectionPass,
    indirect_diffuse_denoising => IndirectDiffuseDenoisingPass,
    indirect_diffuse_resolving => IndirectDiffuseResolvingPass,
    indirect_diffuse_spatial_resampling => IndirectDiffuseSpatialResamplingPass,
    indirect_diffuse_temporal_resampling => IndirectDiffuseTemporalResamplingPass,
    indirect_seeding => IndirectSeedingPass,
    indirect_specular_denoising => IndirectSpecularDenoisingPass,
    indirect_specular_resampling => IndirectSpecularResamplingPass,
    indirect_specular_resolving => IndirectSpecularResolvingPass,
    indirect_tracing => IndirectTracingPass,
    reference_shading => ReferenceShadingPass,
    reference_tracing => ReferenceTracingPass,
]);
