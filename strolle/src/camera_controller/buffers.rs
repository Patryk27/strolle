//! TODO lots of the textures here could use simpler formats

use log::debug;
use spirv_std::glam::UVec2;

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

    pub direct_hits: Texture,
    pub direct_depth: Texture,
    pub direct_gbuffer_d0: Texture,
    pub direct_gbuffer_d1: Texture,

    pub direct_samples: Texture,
    pub direct_colors: DoubleBuffered<Texture>,
    pub direct_initial_samples: StorageBuffer,
    pub direct_temporal_reservoirs: DoubleBuffered<StorageBuffer>,
    pub direct_spatial_reservoirs: DoubleBuffered<StorageBuffer>,

    pub indirect_rays: Texture,
    pub indirect_gbuffer_d0: Texture,
    pub indirect_gbuffer_d1: Texture,
    pub indirect_samples: StorageBuffer,

    pub indirect_diffuse_colors: DoubleBuffered<Texture>,
    pub indirect_diffuse_samples: Texture,
    pub indirect_diffuse_temporal_reservoirs: DoubleBuffered<StorageBuffer>,
    pub indirect_diffuse_spatial_reservoirs: DoubleBuffered<StorageBuffer>,

    pub indirect_specular_colors: DoubleBuffered<Texture>,
    pub indirect_specular_samples: Texture,
    pub indirect_specular_reservoirs: DoubleBuffered<StorageBuffer>,

    pub surface_map: DoubleBuffered<Texture>,
    pub reprojection_map: Texture,
    pub velocity_map: Texture,

    pub reference_hits: StorageBuffer,
    pub reference_rays: StorageBuffer,
    pub reference_colors: Texture,
}

impl CameraBuffers {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
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

        let direct_hits = Texture::builder("direct_hits")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_depth = Texture::builder("direct_depth")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Depth32Float)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_gbuffer_d0 = Texture::builder("direct_gbuffer_d0")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_gbuffer_d1 = Texture::builder("direct_gbuffer_d1")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_samples = Texture::builder("direct_samples")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let direct_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("direct_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let direct_initial_samples = StorageBuffer::new(
            device,
            "direct_initial_samples",
            viewport_buffer_size(camera.viewport.size, 2 * 4 * 4),
        );

        let direct_temporal_reservoirs = DoubleBuffered::<StorageBuffer>::new(
            device,
            "direct_temporal_reservoirs",
            viewport_buffer_size(camera.viewport.size, 3 * 4 * 4),
        );

        let direct_spatial_reservoirs = DoubleBuffered::<StorageBuffer>::new(
            device,
            "direct_spatial_reservoirs",
            viewport_buffer_size(camera.viewport.size, 3 * 4 * 4),
        );

        // ---------------------------------------------------------------------

        let indirect_rays = Texture::builder("indirect_rays")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let indirect_gbuffer_d0 = Texture::builder("indirect_gbuffer_d0")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let indirect_gbuffer_d1 = Texture::builder("indirect_gbuffer_d1")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let indirect_samples = StorageBuffer::new(
            device,
            "indirect_samples",
            viewport_buffer_size(camera.viewport.size, 3 * 4 * 4),
        );

        // ---

        let indirect_diffuse_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("indirect_diffuse_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let indirect_diffuse_samples =
            Texture::builder("indirect_diffuse_samples")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .build(device);

        let indirect_diffuse_temporal_reservoirs =
            DoubleBuffered::<StorageBuffer>::new(
                device,
                "indirect_diffuse_temporal_reservoirs",
                viewport_buffer_size(camera.viewport.size, 4 * 4 * 4),
            );

        let indirect_diffuse_spatial_reservoirs =
            DoubleBuffered::<StorageBuffer>::new(
                device,
                "indirect_diffuse_spatial_reservoirs",
                viewport_buffer_size(camera.viewport.size, 4 * 4 * 4),
            );

        // ---

        let indirect_specular_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("indirect_specular_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let indirect_specular_samples =
            Texture::builder("indirect_specular_samples")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .build(device);

        let indirect_specular_reservoirs = DoubleBuffered::<StorageBuffer>::new(
            device,
            "indirect_specular_reservoirs",
            viewport_buffer_size(camera.viewport.size, 4 * 4 * 4),
        );

        // ---------------------------------------------------------------------

        let surface_map = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("surface_map")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .with_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );

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

        // TODO initialize lazily
        let reference_rays = StorageBuffer::new(
            device,
            "reference_rays",
            viewport_buffer_size(camera.viewport.size, 3 * 4 * 4),
        );

        // TODO initialize lazily
        let reference_hits = StorageBuffer::new(
            device,
            "reference_hits",
            viewport_buffer_size(camera.viewport.size, 2 * 4 * 4),
        );

        // TODO initialize lazily
        let reference_colors = Texture::builder("reference_colors")
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

            direct_depth,
            direct_gbuffer_d0,
            direct_gbuffer_d1,
            direct_hits,

            direct_samples,
            direct_colors,
            direct_initial_samples,
            direct_temporal_reservoirs,
            direct_spatial_reservoirs,

            indirect_rays,
            indirect_gbuffer_d0,
            indirect_gbuffer_d1,
            indirect_samples,

            indirect_diffuse_colors,
            indirect_diffuse_samples,
            indirect_diffuse_temporal_reservoirs,
            indirect_diffuse_spatial_reservoirs,

            indirect_specular_colors,
            indirect_specular_samples,
            indirect_specular_reservoirs,

            surface_map,
            reprojection_map,
            velocity_map,

            reference_hits,
            reference_rays,
            reference_colors,
        }
    }
}

/// Returns the size of a screen-space buffer with given parameters.
fn viewport_buffer_size(viewport_size: UVec2, element_size: usize) -> usize {
    (viewport_size.x as usize) * (viewport_size.y as usize) * element_size
}
