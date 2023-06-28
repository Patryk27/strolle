//! TODO lots of the textures here could use simpler formats

use log::debug;
use spirv_std::glam::UVec2;

use crate::{
    gpu, Camera, DoubleBuffered, MappedUniformBuffer, Texture,
    UnmappedStorageBuffer,
};

#[derive(Debug)]
pub struct CameraBuffers {
    pub camera: MappedUniformBuffer<gpu::Camera>,
    pub past_camera: MappedUniformBuffer<gpu::Camera>,

    pub atmosphere_transmittance_lut: Texture,
    pub atmosphere_scattering_lut: Texture,
    pub atmosphere_sky_lut: Texture,

    pub direct_hits_d0: Texture,
    pub direct_hits_d1: Texture,
    pub direct_hits_d2: Texture,
    pub direct_hits_d3: Texture,
    pub raw_direct_colors: Texture,
    pub direct_colors: DoubleBuffered<Texture>,
    pub direct_initial_samples: UnmappedStorageBuffer,
    pub direct_temporal_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,
    pub direct_spatial_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,

    pub indirect_hits_d0: Texture,
    pub indirect_hits_d1: Texture,
    pub raw_indirect_colors: Texture,
    pub indirect_colors: DoubleBuffered<Texture>,
    pub indirect_initial_samples: UnmappedStorageBuffer,
    pub indirect_temporal_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,
    pub indirect_spatial_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,

    pub surface_map: DoubleBuffered<Texture>,
    pub reprojection_map: Texture,
    pub velocity_map: Texture,
}

impl CameraBuffers {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
        debug!("Initializing camera buffers");

        let camera_uniform =
            MappedUniformBuffer::new(device, "camera", camera.serialize());

        let past_camera =
            MappedUniformBuffer::new(device, "past_camera", camera.serialize());

        // ---------------------------------------------------------------------

        let atmosphere_transmittance_lut =
            Texture::builder("atmosphere_transmittance_lut")
                .with_size(gpu::Atmosphere::TRANSMITTANCE_LUT_RESOLUTION)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_linear_sampling()
                .build(device);

        let atmosphere_scattering_lut =
            Texture::builder("atmosphere_scattering_lut")
                .with_size(gpu::Atmosphere::SCATTERING_LUT_RESOLUTION)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .with_linear_sampling()
                .build(device);

        let atmosphere_sky_lut = Texture::builder("atmosphere_sky_lut")
            .with_size(gpu::Atmosphere::SKY_LUT_RESOLUTION)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .with_linear_sampling()
            .build(device);

        // ---------------------------------------------------------------------

        let direct_hits_d0 = Texture::builder("direct_hits_d0")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_hits_d1 = Texture::builder("direct_hits_d1")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_hits_d2 = Texture::builder("direct_hits_d2")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let direct_hits_d3 = Texture::builder("direct_hits_d3")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        let raw_direct_colors = Texture::builder("raw_direct_colors")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let direct_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("direct_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .add_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let direct_initial_samples = UnmappedStorageBuffer::new(
            device,
            "direct_initial_samples",
            viewport_buffer_size(camera.viewport.size, 4 * 4),
        );

        let direct_temporal_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "direct_temporal_reservoirs",
                viewport_buffer_size(camera.viewport.size, 2 * 4 * 4),
            );

        let direct_spatial_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "direct_spatial_reservoirs",
                viewport_buffer_size(camera.viewport.size, 2 * 4 * 4),
            );

        // ---------------------------------------------------------------------

        let indirect_hits_d0 = Texture::builder("indirect_hits_d0")
            .with_size(camera.viewport.size / 2)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let indirect_hits_d1 = Texture::builder("indirect_hits_d1")
            .with_size(camera.viewport.size / 2)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let raw_indirect_colors = Texture::builder("raw_indirect_colors")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let indirect_colors = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("indirect_colors")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba16Float)
                .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .add_usage(wgpu::TextureUsages::STORAGE_BINDING),
        );

        let indirect_initial_samples = UnmappedStorageBuffer::new(
            device,
            "indirect_initial_samples",
            viewport_buffer_size(camera.viewport.size / 2, 3 * 4 * 4),
        );

        let indirect_temporal_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "indirect_temporal_reservoirs",
                viewport_buffer_size(camera.viewport.size / 2, 4 * 4 * 4),
            );

        let indirect_spatial_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "indirect_spatial_reservoirs",
                viewport_buffer_size(camera.viewport.size / 2, 4 * 4 * 4),
            );

        // ---------------------------------------------------------------------

        let surface_map = DoubleBuffered::<Texture>::new(
            device,
            Texture::builder("surface_map")
                .with_size(camera.viewport.size)
                .with_format(wgpu::TextureFormat::Rgba32Float)
                .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
                .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
                .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        );

        let reprojection_map = Texture::builder("reprojection_map")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .build(device);

        let velocity_map = Texture::builder("reprojection_map")
            .with_size(camera.viewport.size)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .add_usage(wgpu::TextureUsages::TEXTURE_BINDING)
            .add_usage(wgpu::TextureUsages::STORAGE_BINDING)
            .add_usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
            .build(device);

        Self {
            camera: camera_uniform,
            past_camera,

            atmosphere_transmittance_lut,
            atmosphere_scattering_lut,
            atmosphere_sky_lut,

            direct_hits_d0,
            direct_hits_d1,
            direct_hits_d2,
            direct_hits_d3,
            raw_direct_colors,
            direct_colors,
            direct_initial_samples,
            direct_temporal_reservoirs,
            direct_spatial_reservoirs,

            indirect_hits_d0,
            indirect_hits_d1,
            raw_indirect_colors,
            indirect_colors,
            indirect_initial_samples,
            indirect_temporal_reservoirs,
            indirect_spatial_reservoirs,

            surface_map,
            reprojection_map,
            velocity_map,
        }
    }
}

/// Returns the size of a screen-space buffer with given parameters.
fn viewport_buffer_size(viewport_size: UVec2, element_size: usize) -> usize {
    (viewport_size.x as usize) * (viewport_size.y as usize) * element_size
}
