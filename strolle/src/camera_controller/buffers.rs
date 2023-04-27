use std::mem;

use log::debug;
use spirv_std::glam::Vec4;

use crate::{gpu, Camera, MappedUniformBuffer, Texture, UnmappedStorageBuffer};

#[derive(Debug)]
pub struct CameraBuffers {
    pub camera: MappedUniformBuffer<gpu::Camera>,
    pub primary_hits_d0: Texture,
    pub primary_hits_d1: Texture,
    pub primary_hits_d2: Texture,
    pub voxels: UnmappedStorageBuffer,
    pub pending_voxels: UnmappedStorageBuffer,
    pub directs: Texture,
    pub pending_directs: Texture,
    pub indirects: Texture,
    pub pending_indirects: Texture,
    pub normals: Texture,
    pub pending_normals: Texture,
}

impl CameraBuffers {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
        debug!("Initializing camera buffers");

        let camera_uniform = MappedUniformBuffer::new(
            device,
            "strolle_camera",
            mem::size_of::<gpu::Camera>(),
            camera.serialize(),
        );

        let primary_hits_d0 = Texture::new(
            device,
            "strolle_primary_hits_d0",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let primary_hits_d1 = Texture::new(
            device,
            "strolle_primary_hits_d1",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let primary_hits_d2 = Texture::new(
            device,
            "strolle_primary_hits_d2",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba32Float,
        );

        let voxels = UnmappedStorageBuffer::new(
            device,
            "strolle_voxels",
            gpu::VOXELS_MAP_LENGTH * (2 * mem::size_of::<Vec4>()),
        );

        let pending_voxels = UnmappedStorageBuffer::new(
            device,
            "strolle_pending_voxels",
            ((camera.viewport.size.x / 2) * (camera.viewport.size.y / 2))
                as usize
                * (2 * mem::size_of::<Vec4>()),
        );

        let directs = Texture::new(
            device,
            "strolle_directs",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let pending_directs = Texture::new(
            device,
            "strolle_pending_directs",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let indirects = Texture::new(
            device,
            "strolle_indirects",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let pending_indirects = Texture::new(
            device,
            "strolle_pending_indirects",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let normals = Texture::new(
            device,
            "strolle_normals",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        let pending_normals = Texture::new(
            device,
            "strolle_pending_normals",
            camera.viewport.size,
            wgpu::TextureFormat::Rgba16Float,
        );

        Self {
            camera: camera_uniform,
            primary_hits_d0,
            primary_hits_d1,
            primary_hits_d2,
            voxels,
            pending_voxels,
            directs,
            pending_directs,
            indirects,
            pending_indirects,
            normals,
            pending_normals,
        }
    }
}
