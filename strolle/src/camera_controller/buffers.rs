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

    pub direct_hits_d0: Texture,
    pub direct_hits_d1: Texture,
    pub direct_hits_d2: Texture,
    pub direct_colors: DoubleBuffered<Texture>,

    pub indirect_hits_d0: Texture,
    pub indirect_hits_d1: Texture,
    pub raw_indirect_colors: Texture,
    pub indirect_colors: DoubleBuffered<Texture>,
    pub indirect_initial_samples: UnmappedStorageBuffer,
    pub indirect_temporal_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,
    pub indirect_spatial_reservoirs: DoubleBuffered<UnmappedStorageBuffer>,

    pub geometry_map: DoubleBuffered<Texture>,
    pub reprojection_map: Texture,
}

impl CameraBuffers {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
        debug!("Initializing camera buffers");

        // TODO lots of the textures here could use simpler formats

        let camera_uniform = MappedUniformBuffer::new(
            device,
            "strolle_camera",
            camera.serialize(),
        );

        let past_camera = MappedUniformBuffer::new(
            device,
            "strolle_past_camera",
            camera.serialize(),
        );

        // ---------------------------------------------------------------------

        let direct_hits_d0 = Texture::new(
            device,
            "strolle_direct_hits_d0",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let direct_hits_d1 = Texture::new(
            device,
            "strolle_direct_hits_d1",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let direct_hits_d2 = Texture::new(
            device,
            "strolle_direct_hits_d2",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let direct_colors = DoubleBuffered::<Texture>::new(
            device,
            "strolle_direct_colors",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        // ---------------------------------------------------------------------

        let indirect_hits_d0 = Texture::new(
            device,
            "strolle_indirect_hits_d0",
            camera.viewport.size / 2,
            wgpu::TextureFormat::Rgba32Float,
        );

        let indirect_hits_d1 = Texture::new(
            device,
            "strolle_indirect_hits_d1",
            camera.viewport.size / 2,
            wgpu::TextureFormat::Rgba32Float,
        );

        let raw_indirect_colors = Texture::new(
            device,
            "strolle_raw_indirect_colors",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let indirect_colors = DoubleBuffered::<Texture>::new(
            device,
            "strolle_indirect_colors",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let indirect_initial_samples = UnmappedStorageBuffer::new(
            device,
            "strolle_indirect_initial_samples",
            viewport_buffer_size(camera.viewport.size / 2, 3 * 4 * 4),
        );

        let indirect_temporal_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "strolle_indirect_temporal_reservoirs",
                viewport_buffer_size(camera.viewport.size / 2, 4 * 4 * 4),
            );

        let indirect_spatial_reservoirs =
            DoubleBuffered::<UnmappedStorageBuffer>::new(
                device,
                "strolle_indirect_spatial_reservoirs",
                viewport_buffer_size(camera.viewport.size / 2, 4 * 4 * 4),
            );

        // ---------------------------------------------------------------------

        let geometry_map = DoubleBuffered::<Texture>::new(
            device,
            "strolle_geometry_map",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let reprojection_map = Texture::new(
            device,
            "strolle_reprojection_map",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        Self {
            camera: camera_uniform,
            past_camera,

            direct_hits_d0,
            direct_hits_d1,
            direct_hits_d2,
            direct_colors,

            indirect_hits_d0,
            indirect_hits_d1,
            raw_indirect_colors,
            indirect_colors,
            indirect_initial_samples,
            indirect_temporal_reservoirs,
            indirect_spatial_reservoirs,

            geometry_map,
            reprojection_map,
        }
    }
}

/// Returns the size of a screen-space buffer with given parameters.
fn viewport_buffer_size(viewport_size: UVec2, element_size: usize) -> usize {
    (viewport_size.x as usize) * (viewport_size.y as usize) * element_size
}
