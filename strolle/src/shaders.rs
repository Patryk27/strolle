macro_rules! shaders {
    ([ $( $name:ident => $file:literal, )* ]) => {
        #[derive(Debug)]
        pub struct Shaders {
            $( pub $name: wgpu::ShaderModule, )*
        }

        impl Shaders {
            pub fn new(device: &wgpu::Device) -> Self {
                $(
                    log::info!("Initializing shader: {}", stringify!($name));

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
    denoising => "denoising.spv",
    drawing => "drawing.spv",
    raster => "raster.spv",
    ray_shading => "ray-shading.spv",
    voxel_painting => "voxel-painting.spv",
    voxel_shading => "voxel-shading.spv",
    voxel_tracing => "voxel-tracing.spv",
]);
