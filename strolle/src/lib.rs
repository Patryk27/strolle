#![feature(type_alias_impl_trait)]

mod buffers;
mod geometry_bvh;
mod geometry_tris;
mod geometry_uvs;
mod viewport;

use std::sync::Arc;

use spirv_std::glam::UVec2;
pub use strolle_models::*;

pub(crate) use self::buffers::*;
pub use self::geometry_bvh::*;
pub use self::geometry_tris::*;
pub use self::geometry_uvs::*;
pub use self::viewport::*;

pub struct Engine {
    tracer: wgpu::ShaderModule,
    materializer: wgpu::ShaderModule,
    printer: wgpu::ShaderModule,
    geometry_tris: Arc<StorageBuffer<GeometryTris>>,
    geometry_uvs: Arc<StorageBuffer<GeometryUvs>>,
    geometry_bvh: Arc<StorageBuffer<GeometryBvh>>,
    lights: Arc<UniformBuffer<Lights>>,
    materials: Arc<UniformBuffer<Materials>>,
}

impl Engine {
    pub fn new(device: &wgpu::Device) -> Self {
        // TODO support dynamic buffers
        const BUF_SIZE: usize = (128 + 64) * 1024 * 1024;

        log::info!("Initializing");

        let tracer = device.create_shader_module(wgpu::include_spirv!(
            "../../target/tracer.spv"
        ));

        let materializer = device.create_shader_module(wgpu::include_spirv!(
            "../../target/materializer.spv"
        ));

        let printer = device.create_shader_module(wgpu::include_spirv!(
            "../../target/printer.spv"
        ));

        let geometry_tris =
            StorageBuffer::new(device, "strolle_geometry_tris", BUF_SIZE);

        let geometry_uvs =
            StorageBuffer::new(device, "strolle_geometry_uvs", BUF_SIZE);

        let geometry_bvh =
            StorageBuffer::new(device, "strolle_geometry_bvh", 2 * BUF_SIZE);

        let lights = UniformBuffer::new(device, "strolle_lights");
        let materials = UniformBuffer::new(device, "strolle_materials");

        Self {
            tracer,
            materializer,
            printer,
            geometry_tris: Arc::new(geometry_tris),
            geometry_uvs: Arc::new(geometry_uvs),
            geometry_bvh: Arc::new(geometry_bvh),
            lights: Arc::new(lights),
            materials: Arc::new(materials),
        }
    }

    pub fn write_geometry(
        &self,
        queue: &wgpu::Queue,
        geometry_tris: &GeometryTris,
        geometry_uvs: &GeometryUvs,
        geometry_bvh: &GeometryBvh,
    ) {
        self.geometry_tris.write(queue, geometry_tris);
        self.geometry_uvs.write(queue, geometry_uvs);
        self.geometry_bvh.write(queue, geometry_bvh);
    }

    pub fn write_lights(&self, queue: &wgpu::Queue, lights: &Lights) {
        self.lights.write(queue, lights);
    }

    pub fn write_materials(&self, queue: &wgpu::Queue, materials: &Materials) {
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
