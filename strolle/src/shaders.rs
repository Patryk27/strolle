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
    direct_initial_shading,
    direct_raster,
    direct_resolving,
    direct_spatial_resampling,
    direct_temporal_resampling,
    frame_composition,
    frame_reprojection,
    indirect_diffuse_denoising,
    indirect_diffuse_spatial_resampling,
    indirect_diffuse_temporal_resampling,
    indirect_initial_shading,
    indirect_initial_tracing,
    indirect_resolving,
    indirect_specular_denoising,
    indirect_specular_resampling,
    reference_shading,
    reference_tracing,
]);
