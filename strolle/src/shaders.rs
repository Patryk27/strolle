use log::info;

macro_rules! shaders {
    ([ $( $name:ident, )* ]) => {
        #[derive(Debug)]
        pub struct Shaders {
            $( pub $name: (wgpu::ShaderModule, &'static str), )*
        }

        impl Shaders {
            pub fn new(device: &wgpu::Device) -> Self {
                $(
                    info!("Initializing shader: {}", stringify!($name));

                    let module = wgpu::include_spirv!(
                        env!(concat!("strolle_shaders::", stringify!($name), ".path"))
                    );

                    // Safety: fingers crossed™
                    //
                    // We do our best, but our shaders are so array-intensive
                    // that adding the checks decreases performance by 33%, so
                    // it's pretty much a no-go.
                    let module = unsafe {
                        device.create_shader_module_unchecked(module)
                    };

                    let entry_point = env!(concat!("strolle_shaders::", stringify!($name), ".entry_point"));

                    let $name = (module, entry_point);
                )*

                Self {
                    $($name,)*
                }
            }
        }
    };
}

shaders!([
    atmosphere_generate_scattering_lut,
    atmosphere_generate_sky_lut,
    atmosphere_generate_transmittance_lut,
    bvh_heatmap,
    di_resolving,
    di_shading,
    di_spatial_resampling,
    di_temporal_resampling,
    frame_composition_fs,
    frame_composition_vs,
    frame_denoising_estimate_variance,
    frame_denoising_reproject,
    frame_denoising_wavelet,
    frame_reprojection,
    gi_diff_resolving,
    gi_diff_spatial_resampling_fast,
    gi_diff_spatial_resampling_slow,
    gi_diff_temporal_resampling,
    gi_shading,
    gi_spec_resampling,
    gi_spec_resolving,
    gi_tracing,
    prim_raster_fs,
    prim_raster_vs,
    ref_shading,
    ref_tracing,
]);
