use log::info;

macro_rules! shaders {
    ([ $( $name:ident => $file:literal, )* ]) => {
        #[derive(Debug)]
        pub struct Shaders {
            $( pub $name: wgpu::ShaderModule, )*
        }

        impl Shaders {
            pub fn new(device: &wgpu::Device) -> Self {
                $(
                    info!("Initializing shader: {}", stringify!($name));

                    let $name = device.create_shader_module(wgpu::include_spirv!(concat!(
                        "../../target/",
                        $file,
                    )));
                )*

                Self {
                    $($name,)*
                }
            }
        }
    };
}

shaders!([
    atmosphere => "atmosphere.spv",
    direct_raster => "direct-raster.spv",
    direct_shading => "direct-shading.spv",
    direct_tracing => "direct-tracing.spv",
    indirect_denoising => "indirect-denoising.spv",
    indirect_initial_shading => "indirect-initial-shading.spv",
    indirect_initial_tracing => "indirect-initial-tracing.spv",
    indirect_resolving => "indirect-resolving.spv",
    indirect_spatial_resampling => "indirect-spatial-resampling.spv",
    indirect_temporal_resampling => "indirect-temporal-resampling.spv",
    output_drawing => "output-drawing.spv",
    reprojection => "reprojection.spv",
]);
