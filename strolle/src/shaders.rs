use log::info;

macro_rules! shaders {
    ([ $( $name:ident, )* ]) => {
        #[derive(Debug)]
        pub struct Shaders {
            $( pub $name: wgpu::ShaderModule, )*
        }

        impl Shaders {
            pub fn new(device: &wgpu::Device) -> Self {
                $(
                    info!("Initializing shader: {}", stringify!($name));

                    let $name = device.create_shader_module(wgpu::include_spirv!(
                        env!(concat!("strolle_", stringify!($name), "_shader.spv"))
                    ));
                )*

                Self {
                    $($name,)*
                }
            }
        }
    };
}

shaders!([
    atmosphere,
    bvh_heatmap,
    direct_denoising,
    direct_raster,
    direct_resolving,
    direct_shading,
    direct_spatial_resampling,
    direct_temporal_resampling,
    direct_validation,
    frame_composition,
    frame_reprojection,
    indirect_diffuse_denoising,
    indirect_diffuse_resolving,
    indirect_diffuse_spatial_resampling,
    indirect_diffuse_temporal_resampling,
    indirect_shading,
    indirect_specular_denoising,
    indirect_specular_resampling,
    indirect_specular_resolving,
    indirect_tracing,
    reference_shading,
    reference_tracing,
]);
