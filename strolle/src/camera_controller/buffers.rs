use log::debug;
use strolle_gpu::DiReservoirData;
use crate::{
    gpu, Camera, DoubleBuffered, MappedUniformBuffer, StorageBuffer, Texture,
};
use crate::utils::ToGpu;

#[derive(Debug)]
pub struct CameraBuffers {
    pub curr_camera: MappedUniformBuffer<gpu::Camera>,
    pub prev_camera: MappedUniformBuffer<gpu::Camera>,

    pub atmosphere_transmittance_lut: Texture,
    pub atmosphere_scattering_lut: Texture,
    pub atmosphere_sky_lut: Texture,

    pub prim_depth: Texture,
    pub prim_gbuffer_d0: DoubleBuffered<Texture>,
    pub prim_gbuffer_d1: DoubleBuffered<Texture>,

    pub reprojection_map: Texture,
    pub velocity_map: Texture,

    pub di_reservoirs: [StorageBuffer; 3],

    pub di_diff_samples: Texture,
    pub di_diff_prev_colors: Texture,
    pub di_diff_curr_colors: Texture,
    pub di_diff_moments: DoubleBuffered<Texture>,
    pub di_diff_stash: Texture,

    pub di_spec_samples: Texture,

    pub gi_d0: Texture,
    pub gi_d1: Texture,
    pub gi_d2: Texture,
    pub gi_reservoirs: [StorageBuffer; 4],

    pub gi_diff_samples: Texture,
    pub gi_diff_prev_colors: Texture,
    pub gi_diff_curr_colors: Texture,
    pub gi_diff_moments: DoubleBuffered<Texture>,
    pub gi_diff_stash: Texture,

    pub gi_spec_samples: Texture,

    pub ref_hits: StorageBuffer,
    pub ref_rays: StorageBuffer,
    pub ref_colors: Texture,
}

impl CameraBuffers {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
        // Returns the size of a screen-space buffer with given parameters
        let viewport_buffer_size = |element_size| {
            (camera.viewport.size.x as usize)
                * (camera.viewport.size.y as usize)
                * element_size
        };

        // ---

        debug!("Initializing camera buffers");

        let camera_uniform =
            MappedUniformBuffer::new(device, "camera", camera.serialize());

        let prev_camera =
            MappedUniformBuffer::new(device, "prev_camera", camera.serialize());

        // ---------------------------------------------------------------------

        let atmosphere_transmittance_lut =
            Texture::builder("atmosphere_transmittance_lut")
                .with_size(gpu::Atmosphere::TRANSMITTANCE_LUT_RESOLUTION)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_linear_filtering_sampler()
                .build(device);

        let atmosphere_scattering_lut =
            Texture::builder("atmosphere_scattering_lut")
                .with_size(gpu::Atmosphere::SCATTERING_LUT_RESOLUTION)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_linear_filtering_sampler()
                .build(device);

        let atmosphere_sky_lut = Texture::builder("atmosphere_sky_lut")
            .with_size(gpu::Atmosphere::SKY_LUT_RESOLUTION)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_linear_filtering_sampler()
            .build(device);

        // ---------------------------------------------------------------------

        let prim_depth = Texture::builder("prim_depth")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Depth32Float)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let prim_gbuffer_d0 = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("prim_gbuffer_d0")
                .with_size(camera.viewport.size.to_gpu())
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );

        let prim_gbuffer_d1 = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("prim_gbuffer_d1")
                .with_size(camera.viewport.size.to_gpu())
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );

        // ---------------------------------------------------------------------

        let reprojection_map = Texture::builder("reprojection_map")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let velocity_map = Texture::builder("velocity_map")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        // ---------------------------------------------------------------------

        let element_size = std::mem::size_of::<DiReservoirData>();
        let element_size_round = (element_size as f32 / 32.0).ceil() as usize * 32;
        let di_reservoirs = [0, 1, 2].map(|idx| {
            StorageBuffer::new(
                device,
                format!("di_reservoir_{}", idx),
                viewport_buffer_size(element_size_round),
            )
        });

        // ---

        let di_diff_samples = Texture::builder("di_diff_samples")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_prev_colors = Texture::builder("di_diff_prev_colors")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_curr_colors = Texture::builder("di_diff_curr_colors")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_moments = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("di_diff_moments")
                .with_size(camera.viewport.size.to_gpu())
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let di_diff_stash = Texture::builder("di_diff_stash")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---

        let di_spec_samples = Texture::builder("di_spec_samples")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---------------------------------------------------------------------

        let gi_d0 = Texture::builder("gi_d0")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_d1 = Texture::builder("gi_d1")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_d2 = Texture::builder("gi_d2")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_reservoirs = [0, 1, 2, 3].map(|idx| {
            StorageBuffer::new(
                device,
                format!("gi_reservoir_{}", idx),
                viewport_buffer_size(4 * 4 * 4),
            )
        });

        // ---

        let gi_diff_samples = Texture::builder("gi_diff_samples")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_diff_prev_colors = Texture::builder("gi_diff_prev_colors")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_diff_curr_colors = Texture::builder("gi_diff_curr_colors")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_diff_moments = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("gi_diff_moments")
                .with_size(camera.viewport.size.to_gpu())
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let gi_diff_stash = Texture::builder("gi_diff_stash")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---

        let gi_spec_samples = Texture::builder("gi_spec_samples")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---------------------------------------------------------------------

        // TODO initialize lazily
        let ref_rays = StorageBuffer::new(
            device,
            "ref_rays",
            viewport_buffer_size(3 * 4 * 4),
        );

        // TODO initialize lazily
        let ref_hits = StorageBuffer::new(
            device,
            "ref_hits",
            viewport_buffer_size(2 * 4 * 4),
        );

        // TODO initialize lazily
        let ref_colors = Texture::builder("ref_colors")
            .with_size(camera.viewport.size.to_gpu())
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---------------------------------------------------------------------

        Self {
            curr_camera: camera_uniform,
            prev_camera,

            atmosphere_transmittance_lut,
            atmosphere_scattering_lut,
            atmosphere_sky_lut,

            prim_depth,
            prim_gbuffer_d0,
            prim_gbuffer_d1,

            reprojection_map,
            velocity_map,

            di_reservoirs,

            di_diff_samples,
            di_diff_prev_colors,
            di_diff_curr_colors,
            di_diff_moments,
            di_diff_stash,

            di_spec_samples,

            gi_d0,
            gi_d1,
            gi_d2,
            gi_reservoirs,

            gi_diff_samples,
            gi_diff_prev_colors,
            gi_diff_curr_colors,
            gi_diff_moments,
            gi_diff_stash,

            gi_spec_samples,

            ref_hits,
            ref_rays,
            ref_colors,
        }
    }
}
