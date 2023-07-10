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

                    let $name = device.create_shader_module(wgpu::include_spirv!(env!($file)));
                )*

                Self {
                    $($name,)*
                }
            }
        }
    };
}

shaders!([
    atmosphere => "strolle_atmosphere_shader.spv",
    bvh_heatmap => "strolle_bvh_heatmap_shader.spv",
    direct_denoising => "strolle_direct_denoising_shader.spv",
    direct_initial_shading => "strolle_direct_initial_shading_shader.spv",
    direct_raster => "strolle_direct_raster_shader.spv",
    direct_resolving => "strolle_direct_resolving_shader.spv",
    direct_secondary_tracing => "strolle_direct_secondary_tracing_shader.spv",
    direct_spatial_resampling => "strolle_direct_spatial_resampling_shader.spv",
    direct_temporal_resampling => "strolle_direct_temporal_resampling_shader.spv",
    indirect_denoising => "strolle_indirect_denoising_shader.spv",
    indirect_initial_shading => "strolle_indirect_initial_shading_shader.spv",
    indirect_initial_tracing => "strolle_indirect_initial_tracing_shader.spv",
    indirect_resolving => "strolle_indirect_resolving_shader.spv",
    indirect_spatial_resampling => "strolle_indirect_spatial_resampling_shader.spv",
    indirect_temporal_resampling => "strolle_indirect_temporal_resampling_shader.spv",
    output_drawing => "strolle_output_drawing_shader.spv",
    reprojection => "strolle_reprojection_shader.spv",
]);
