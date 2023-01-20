#![feature(array_chunks)]
#![feature(type_alias_impl_trait)]

mod buffers;
mod bvh;
mod camera;
mod image;
mod images;
mod instance;
mod instances;
mod light;
mod lights;
mod material;
mod materials;
mod mesh;
mod meshes;
mod shaders;
mod triangle;
mod triangles;
mod viewport;

use std::fmt::Debug;
use std::hash::Hash;
use std::mem;
use std::time::Instant;

use spirv_std::glam::UVec2;
use strolle_models as gpu;

pub(crate) use self::buffers::*;
pub(crate) use self::bvh::*;
pub use self::camera::*;
pub use self::image::*;
pub(crate) use self::images::*;
pub use self::instance::*;
pub(crate) use self::instances::*;
pub use self::light::*;
pub(crate) use self::lights::*;
pub use self::material::*;
pub(crate) use self::materials::*;
pub use self::mesh::*;
pub(crate) use self::meshes::*;
pub(crate) use self::shaders::*;
pub use self::triangle::*;
pub(crate) use self::triangles::*;
pub use self::viewport::*;

#[derive(Debug)]
pub struct Engine<P>
where
    P: Params,
{
    shaders: Shaders,
    meshes: Meshes<P>,
    instances: Instances<P>,
    triangles: Triangles<P>,
    bvh: Bvh,
    lights: Lights<P>,
    images: Images<P>,
    materials: Materials<P>,
    world: MappedUniformBuffer<gpu::World>,
    viewports: Vec<WeakViewport>,
    has_dirty_images: bool,
    has_dirty_materials: bool,
}

impl<P> Engine<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        log::info!("Initializing");

        Self {
            shaders: Shaders::new(device),
            meshes: Meshes::default(),
            instances: Instances::default(),
            triangles: Triangles::new(device),
            bvh: Bvh::new(device),
            lights: Lights::new(device),
            images: Images::new(device),
            materials: Materials::new(device),
            world: MappedUniformBuffer::new_default(device, "strolle_world"),
            viewports: Default::default(),
            has_dirty_images: Default::default(),
            has_dirty_materials: Default::default(),
        }
    }

    pub fn add_mesh(&mut self, mesh_handle: P::MeshHandle, mesh: Mesh) {
        self.meshes.add(mesh_handle, mesh);
    }

    pub fn remove_mesh(&mut self, mesh_handle: &P::MeshHandle) {
        self.meshes.remove(mesh_handle);
    }

    pub fn add_material(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material<P>,
    ) {
        self.materials.add(material_handle, material);
        self.has_dirty_materials = true;
    }

    pub fn remove_material(&mut self, material_handle: &P::MaterialHandle) {
        self.materials.remove(material_handle);
        self.has_dirty_materials = true;
    }

    pub fn add_image(
        &mut self,
        image_handle: P::ImageHandle,
        image_texture: P::ImageTexture,
        image_sampler: P::ImageSampler,
    ) {
        self.images.add(image_handle, image_texture, image_sampler);
        self.has_dirty_images = true;
    }

    pub fn remove_image(&mut self, image_handle: &P::ImageHandle) {
        self.images.remove(image_handle);
        self.has_dirty_images = true;
    }

    pub fn add_instance(
        &mut self,
        instance_handle: P::InstanceHandle,
        instance: Instance<P>,
    ) {
        self.instances.add(instance_handle, instance);
    }

    pub fn remove_instance(&mut self, instance_handle: &P::InstanceHandle) {
        self.instances.remove(instance_handle);
        self.triangles.remove(instance_handle);
    }

    pub fn add_light(
        &mut self,
        light_handle: P::LightHandle,
        light: gpu::Light,
    ) {
        self.lights.add(light_handle, light);
    }

    pub fn remove_light(&mut self, light_handle: &P::LightHandle) {
        self.lights.remove(light_handle);
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let tt = Instant::now();

        self.world.triangle_count = self.triangles.len() as u32;
        self.world.light_count = self.lights.len();
        self.world.flush(queue);

        if self.has_dirty_images || self.has_dirty_materials {
            // If materials have changed, we have to rebuild them so that
            // their CPU-representations are transformed into the GPU-ones.
            //
            // If images have changed, we have to rebuild materials so that
            // their texture ids are up-to-date (e.g. removing a texture can
            // shift other texture ids by -1, which we have to account for in
            // the materials).

            self.materials.refresh(&self.images);
        }

        if self.has_dirty_images {
            // If images have changed, we have to rebuild the pipelines so that
            // new images & samplers are propagated to `wgpu::PipelineLayout`
            // and `wgpu::ComputePipeline`.
            //
            // This is a somewhat heavy & awkward thing to do, and unfortunately
            // there seems to be no other way - creating a pipeline sets its
            // layout in stone and (as compared to buffers) images & samplers
            // cannot be updated dynamically.

            let viewports = mem::take(&mut self.viewports);

            self.viewports = viewports
                .into_iter()
                .filter(|viewport| {
                    if let Some(viewport) = viewport.upgrade() {
                        viewport.rebuild(self, device);
                        true
                    } else {
                        // This viewport has been dropped
                        false
                    }
                })
                .collect();
        }

        let instances_changed =
            self.instances.refresh(&self.meshes, &mut self.triangles);

        if instances_changed {
            self.bvh
                .refresh(&self.instances, &self.materials, &self.triangles);
        }

        self.has_dirty_images = false;
        self.has_dirty_materials = false;

        // ---

        self.bvh.flush(queue);
        self.triangles.flush(queue);
        self.lights.flush(queue);
        self.materials.flush(queue);

        log::trace!("write() took {:?}", tt.elapsed());
    }

    pub fn create_viewport(
        &mut self,
        device: &wgpu::Device,
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
        camera: Camera,
    ) -> Viewport {
        let viewport = Viewport::new(self, device, pos, size, format, camera);

        self.viewports.push(viewport.downgrade());

        viewport
    }
}

pub trait Params
where
    Self: Clone + Debug,
{
    type ImageHandle: Eq + Hash + Clone + Debug;
    type ImageSampler: ImageSampler + Debug;
    type ImageTexture: ImageTexture + Debug;
    type InstanceHandle: Eq + Hash + Clone + Debug;
    type LightHandle: Eq + Hash + Clone + Debug;
    type MaterialHandle: Eq + Hash + Clone + Debug;
    type MeshHandle: Eq + Hash + Clone + Debug;
}
