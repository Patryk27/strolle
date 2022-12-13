#![feature(type_alias_impl_trait)]

mod buffers;
mod geometry_bvh;
mod geometry_tris;
mod geometry_uvs;
mod viewport;

use std::sync::Arc;

use spirv_std::glam::UVec2;
pub use strolle_raytracer_models::*;

pub(crate) use self::buffers::*;
pub use self::geometry_bvh::*;
pub use self::geometry_tris::*;
pub use self::geometry_uvs::*;
pub use self::viewport::*;

pub struct Engine {
    raytracer: wgpu::ShaderModule,
    renderer: wgpu::ShaderModule,
    geometry_tris: Arc<StorageBuffer<GeometryTris>>,
    geometry_uvs: Arc<StorageBuffer<GeometryUvs>>,
    geometry_bvh: Arc<StorageBuffer<GeometryBvh>>,
    lights: Arc<UniformBuffer<Lights>>,
    materials: Arc<UniformBuffer<Materials>>,
}

impl Engine {
    pub fn new(device: &wgpu::Device) -> Self {
        // TODO support dynamic buffers
        const BUF_SIZE: usize = 32 * 1024 * 1024;

        log::info!("Initializing");

        let raytracer = device.create_shader_module(wgpu::include_spirv!(
            "../../target/raytracer.spv"
        ));

        let renderer = device.create_shader_module(wgpu::include_spirv!(
            "../../target/renderer.spv"
        ));

        let geometry_tris =
            StorageBuffer::new(device, "strolle_geometry_tris", BUF_SIZE);

        let geometry_uvs =
            StorageBuffer::new(device, "strolle_geometry_uvs", BUF_SIZE);

        let geometry_bvh =
            StorageBuffer::new(device, "strolle_geometry_bvh", BUF_SIZE);

        let lights = UniformBuffer::new(device, "strolle_lights");
        let materials = UniformBuffer::new(device, "strolle_materials");

        Self {
            raytracer,
            renderer,
            geometry_tris: Arc::new(geometry_tris),
            geometry_uvs: Arc::new(geometry_uvs),
            geometry_bvh: Arc::new(geometry_bvh),
            lights: Arc::new(lights),
            materials: Arc::new(materials),
        }
    }

    pub fn submit(
        &self,
        queue: &wgpu::Queue,
        geometry_tris: &GeometryTris,
        geometry_uvs: &GeometryUvs,
        geometry_bvh: &GeometryBvh,
        lights: &Lights,
        materials: &Materials,
    ) {
        self.geometry_tris.write(queue, geometry_tris);
        self.geometry_uvs.write(queue, geometry_uvs);
        self.geometry_bvh.write(queue, geometry_bvh);
        self.lights.write(queue, lights);
        self.materials.write(queue, materials);
    }

    pub fn create_viewport(
        &self,
        device: &wgpu::Device,
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
    ) -> Viewport {
        Viewport::new(self, device, pos, size, format)
    }
}
