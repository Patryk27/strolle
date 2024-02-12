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
    di_resample_spatial_s0,
    di_resample_spatial_s1,
    di_resample_temporal,
    di_resolve,
    di_sample,
    frame_compose_fs,
    frame_compose_vs,
    frame_denoise_estimate_variance,
    frame_denoise_reproject,
    frame_denoise_wavelet,
    frame_reproject,
    gi_prepare,
    gi_resample_spatial_approx,
    gi_resample_spatial_exact,
    gi_resample_temporal,
    gi_resolve,
    gi_sample,
    prim_raster_fs,
    prim_raster_vs,
    ref_shade,
    ref_trace,
    rt_intersect,
]);
