use log::debug;

use crate::{
    gpu, Camera, DoubleBuffered, MappedUniformBuffer, StorageBuffer, Texture,
};

#[derive(Debug)]
pub struct CameraBuffers {
    pub camera: MappedUniformBuffer<gpu::Camera>,
    pub prev_camera: MappedUniformBuffer<gpu::Camera>,

    pub atmosphere_transmittance_lut: Texture,
    pub atmosphere_scattering_lut: Texture,
    pub atmosphere_sky_lut: Texture,

    pub prim_depth: Texture,
    pub prim_gbuffer_d0: Texture,
    pub prim_gbuffer_d1: Texture,
    pub prim_surface_map: DoubleBuffered<Texture>,

    pub reprojection_map: Texture,
    pub velocity_map: Texture,

    pub di_prev_reservoirs: StorageBuffer,
    pub di_curr_reservoirs: StorageBuffer,
    pub di_next_reservoirs: StorageBuffer,

    pub di_diff_samples: Texture,
    pub di_diff_prev_colors: Texture,
    pub di_diff_curr_colors: Texture,
    pub di_diff_moments: DoubleBuffered<Texture>,
    pub di_diff_stash: Texture,

    pub gi_rays: Texture,
    pub gi_gbuffer_d0: Texture,
    pub gi_gbuffer_d1: Texture,
    pub gi_samples: StorageBuffer,

    pub gi_diff_reservoirs: [StorageBuffer; 4],
    pub gi_diff_samples: Texture,
    pub gi_diff_colors: DoubleBuffered<Texture>,

    pub gi_spec_samples: Texture,
    pub gi_spec_reservoirs: DoubleBuffered<StorageBuffer>,

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
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Depth32Float)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let prim_gbuffer_d0 = Texture::builder("prim_gbuffer_d0")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let prim_gbuffer_d1 = Texture::builder("prim_gbuffer_d1")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let prim_surface_map = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("prim_surface_map")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );

        // ---------------------------------------------------------------------

        let reprojection_map = Texture::builder("reprojection_map")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let velocity_map = Texture::builder("velocity_map")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        // ---------------------------------------------------------------------

        let di_prev_reservoirs = StorageBuffer::new(
            device,
            "di_prev_reservoirs",
            viewport_buffer_size(2 * 4 * 4),
        );

        let di_curr_reservoirs = StorageBuffer::new(
            device,
            "di_curr_reservoirs",
            viewport_buffer_size(2 * 4 * 4),
        );

        let di_next_reservoirs = StorageBuffer::new(
            device,
            "di_next_reservoirs",
            viewport_buffer_size(2 * 4 * 4),
        );

        // ---

        let di_diff_samples = Texture::builder("di_diff_samples")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_prev_colors = Texture::builder("di_diff_prev_colors")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_curr_colors = Texture::builder("di_diff_curr_colors")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let di_diff_moments = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("di_diff_moments")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let di_diff_stash = Texture::builder("di_diff_stash")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---------------------------------------------------------------------

        let gi_rays = Texture::builder("gi_rays")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_gbuffer_d0 = Texture::builder("gi_gbuffer_d0")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_gbuffer_d1 = Texture::builder("gi_gbuffer_d1")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_samples = StorageBuffer::new(
            device,
            "gi_samples",
            viewport_buffer_size(3 * 4 * 4),
        );

        // ---

        let gi_diff_reservoirs = [0, 1, 2, 3].map(|idx| {
            StorageBuffer::new(
                device,
                format!("gi_diff_reservoirs_{}", idx),
                viewport_buffer_size(4 * 4 * 4),
            )
        });

        let gi_diff_samples = Texture::builder("gi_diff_samples")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_diff_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("gi_diff_prev_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        // ---

        let gi_spec_samples = Texture::builder("gi_spec_samples")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let gi_spec_reservoirs = DoubleBuffered::<StorageBuffer>::new(
            device,
            "gi_spec_reservoirs",
            viewport_buffer_size(4 * 4 * 4),
        );

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
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        // ---------------------------------------------------------------------

        Self {
            camera: camera_uniform,
            prev_camera,

            atmosphere_transmittance_lut,
            atmosphere_scattering_lut,
            atmosphere_sky_lut,

            prim_depth,
            prim_gbuffer_d0,
            prim_gbuffer_d1,
            prim_surface_map,

            reprojection_map,
            velocity_map,

            di_prev_reservoirs,
            di_curr_reservoirs,
            di_next_reservoirs,

            di_diff_samples,
            di_diff_prev_colors,
            di_diff_curr_colors,
            di_diff_moments,
            di_diff_stash,

            gi_rays,
            gi_gbuffer_d0,
            gi_gbuffer_d1,
            gi_samples,

            gi_diff_reservoirs,
            gi_diff_samples,
            gi_diff_colors,

            gi_spec_samples,
            gi_spec_reservoirs,

            ref_hits,
            ref_rays,
            ref_colors,
        }
    }
}
