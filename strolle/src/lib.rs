//! Strolle, an experimental real-time renderer that supports global
//! illumination.
//!
//! # Usage
//!
//! If you're using Bevy, please take a look at the `bevy-strolle` crate that
//! provides a seamless integration with Bevy.
//!
//! It's also possible to uses Strolle outside of Bevy, as the low-level
//! interface requires only `wgpu` - there is no tutorial for that just yet,
//! though.
//!
//! # Definitions
//!
//! TODO

#![feature(hash_raw_entry)]
#![feature(once_cell)]
#![feature(option_result_contains)]

mod buffers;
mod bvh;
mod camera;
mod camera_controller;
mod camera_controllers;
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
mod utils;

use std::fmt::Debug;
use std::hash::Hash;
use std::mem;
use std::time::Instant;

use log::{info, trace, warn};
pub(self) use strolle_models as gpu;

pub(self) use self::buffers::*;
pub(self) use self::bvh::*;
pub use self::camera::*;
pub(self) use self::camera_controller::*;
pub(self) use self::camera_controllers::*;
pub use self::image::*;
pub(self) use self::images::*;
pub use self::instance::*;
pub(self) use self::instances::*;
pub use self::light::*;
pub(self) use self::lights::*;
pub use self::material::*;
pub(self) use self::materials::*;
pub use self::mesh::*;
pub(self) use self::meshes::*;
pub(self) use self::shaders::*;
pub use self::triangle::*;
pub(self) use self::triangles::*;

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
    cameras: CameraControllers<P>,
    pending_events: Vec<Event<P>>,
}

impl<P> Engine<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        info!("Initializing");

        Self {
            shaders: Shaders::new(device),
            meshes: Meshes::default(),
            instances: Instances::default(),
            triangles: Triangles::new(device),
            bvh: Bvh::new(device),
            lights: Lights::new(device),
            images: Images::new(device),
            materials: Materials::new(device),
            world: MappedUniformBuffer::new(
                device,
                "strolle_world",
                Default::default(),
            ),
            cameras: Default::default(),
            pending_events: Default::default(),
        }
    }

    /// Creates a mesh¹ or updates the existing one.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_mesh(&mut self, mesh_handle: P::MeshHandle, mesh: Mesh) {
        self.meshes.add(mesh_handle, mesh);
    }

    /// Removes a mesh¹.
    ///
    /// Note that removing a mesh doesn't automatically remove instances¹ that
    /// refer to that mesh.
    ///
    /// That is to say, if you add an instance with mesh A, and then remove this
    /// mesh, the instance will remain in the world - it will not be rendered
    /// though, waiting for the mesh to appear again.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_mesh(&mut self, mesh_handle: &P::MeshHandle) {
        self.meshes.remove(mesh_handle);
    }

    /// Creates a material¹ or updates the existing one.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_material(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material<P>,
    ) {
        self.materials.add(material_handle.clone(), material);

        self.pending_events
            .push(Event::MaterialChanged(material_handle));
    }

    /// Returns whether material¹ with given handle exists or not.
    ///
    /// ¹ see the module-level comment for details
    pub fn has_material(&self, material_handle: &P::MaterialHandle) -> bool {
        self.materials.has(material_handle)
    }

    /// Removes a material¹.
    ///
    /// Note that removing a material doesn't automatically remove instances¹
    /// that refer to that material.
    ///
    /// That is to say, if you add an instance with material A, and then remove
    /// this material, the instance will remain in the world - it will not be
    /// rendered though, waiting for the material to appear again.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_material(&mut self, material_handle: &P::MaterialHandle) {
        self.materials.remove(material_handle);

        self.pending_events
            .push(Event::MaterialRemoved(material_handle.clone()));
    }

    /// Creates an image¹ or updates the existing one.
    ///
    /// Note that updating image's pixels doesn't require recalling this
    /// function², but changing the image's format, its resolution or sampler
    /// does.
    ///
    /// Also, at the moment Strolle supports only single-sampled 2D textures
    /// with a non-filterable samplers; attaching other kind of texture and/or
    /// sampler will crash the renderer.
    ///
    /// ¹ see the module-level comment for details
    /// ² and it's not recommended due to potential performance pitfalls
    pub fn add_image(
        &mut self,
        image_handle: P::ImageHandle,
        image_texture: P::ImageTexture,
        image_sampler: P::ImageSampler,
    ) {
        self.images
            .add(image_handle.clone(), image_texture, image_sampler);

        self.pending_events.push(Event::ImageChanged(image_handle));
    }

    /// Returns whether image¹ with given handle exists or not.
    ///
    /// ¹ see the module-level comment for details
    pub fn has_image(&self, image_handle: &P::ImageHandle) -> bool {
        self.images.has(image_handle)
    }

    /// Removes an image¹.
    ///
    /// Note that removing an image doesn't automatically remove materials¹ that
    /// refer to that image.
    ///
    /// That is to say, if you add a material with image A, and then remove this
    /// image, the material will remain in the world - instances related to that
    /// material will not be rendered though, waiting for the image to appear
    /// again.
    ///
    /// (or, well, waiting for the material to get unbound from the image etc.)
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_image(&mut self, image_handle: &P::ImageHandle) {
        self.images.remove(image_handle);

        self.pending_events
            .push(Event::ImageRemoved(image_handle.clone()));
    }

    /// Creates an instance¹ or updates the existing one.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_instance(
        &mut self,
        instance_handle: P::InstanceHandle,
        instance: Instance<P>,
    ) {
        self.instances.add(instance_handle, instance);
    }

    /// Removes an instance¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_instance(&mut self, instance_handle: &P::InstanceHandle) {
        self.instances.remove(instance_handle);
        self.triangles.remove(instance_handle);
    }

    /// Creates a light or updates the existing one¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_light(
        &mut self,
        light_handle: P::LightHandle,
        light: gpu::Light,
    ) {
        self.lights.add(light_handle, light);
    }

    /// Removes a light¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_light(&mut self, light_handle: &P::LightHandle) {
        self.lights.remove(light_handle);
    }

    /// Removes all lights¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_all_lights(&mut self) {
        self.lights.clear();
    }

    /// Sends all changes to the GPU and prepares it for the upcoming frame.
    ///
    /// This function must be called before each frame, i.e. before invoking
    /// [`Self::render_camera()`].
    ///
    /// (if you have multiple cameras, calling this function just once is
    /// enough.)
    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let tt = Instant::now();

        let any_image_or_material_modified =
            self.pending_events.iter().any(|event| {
                matches!(
                    event,
                    Event::MaterialChanged(_)
                        | Event::MaterialRemoved(_)
                        | Event::ImageChanged(_)
                        | Event::ImageRemoved(_)
                )
            });

        if any_image_or_material_modified {
            self.materials.refresh(&self.images);
        }

        // ---

        let any_instance_modifed =
            self.instances.refresh(&self.meshes, &mut self.triangles);

        if any_instance_modifed {
            self.bvh
                .refresh(&self.instances, &self.materials, &self.triangles);
        }

        // ---

        let any_buffer_reallocated = false
            | self.bvh.flush(device, queue).reallocated
            | self.triangles.flush(device, queue).reallocated
            | self.lights.flush(device, queue).reallocated
            | self.materials.flush(device, queue).reallocated;

        if any_buffer_reallocated {
            self.pending_events.push(Event::BufferReallocated);
        }

        // ---

        self.world.light_count = self.lights.len();

        if let Some(min_aabb) = self.triangles.bounding_box().min_opt() {
            let new_min_aabb = self.world.min_aabb.min(min_aabb);

            if self.world.min_aabb != new_min_aabb {
                warn!(
                    "World's minimum bounding box has changed ({:?} => {:?}); \
                     irradiance cache will be invalidated",
                    self.world.min_aabb, new_min_aabb,
                );

                self.world.min_aabb = new_min_aabb;
            }
        }

        self.world.flush(queue);

        // ---

        let mut any_image_modified = false;

        for event in mem::take(&mut self.pending_events) {
            trace!("Processing event: {event:?}");

            let mut cameras = mem::take(&mut self.cameras);

            for camera in cameras.iter_mut() {
                match &event {
                    Event::BufferReallocated => {
                        camera.on_buffers_reallocated(self, device);
                    }

                    Event::ImageChanged(image_handle) => {
                        camera.on_image_changed(self, device, image_handle);
                        any_image_modified = true;
                    }

                    Event::ImageRemoved(image_handle) => {
                        camera.on_image_removed(self, device, image_handle);
                        any_image_modified = true;
                    }

                    Event::MaterialChanged(material_handle) => {
                        camera.on_material_changed(
                            self,
                            device,
                            material_handle,
                        );
                    }

                    Event::MaterialRemoved(material_handle) => {
                        camera.on_material_removed(material_handle);
                    }
                }
            }

            self.cameras = cameras;
        }

        if any_image_modified {
            let mut cameras = mem::take(&mut self.cameras);

            for camera in cameras.iter_mut() {
                camera.on_images_modified(self, device);
            }

            self.cameras = cameras;
        }

        for camera in self.cameras.iter_mut() {
            camera.flush(queue);
        }

        // ---

        utils::metric("flush", tt);
    }

    /// Creates a new camera¹ that can be used to render the world.
    ///
    /// Note that this is a pretty heavy operation that allocates per-camera
    /// buffers etc., and so it's expected that you only call this function when
    /// necessary (not, say, each frame).
    ///
    /// ¹ see the module-level comment for details
    pub fn create_camera(
        &mut self,
        device: &wgpu::Device,
        camera: Camera,
    ) -> CameraHandle {
        self.cameras
            .add(CameraController::new(self, device, camera))
    }

    /// Updates camera¹, changing its mode, position, size etc.
    ///
    /// ¹ see the module-level comment for details
    pub fn update_camera(
        &mut self,
        device: &wgpu::Device,
        handle: CameraHandle,
        camera: Camera,
    ) {
        let mut cameras = mem::take(&mut self.cameras);

        cameras.get_mut(handle).update(self, device, camera);

        self.cameras = cameras;
    }

    /// Renders camera¹ to texture.
    ///
    /// Texture's format must be the same as the format given to
    /// [`Self::create_camera()`] and - if any changes to the world have been
    /// made - [`Self::flush()`] must've been already called (otherwise the
    /// renderer might panic, draw invalid meshes, stuff like that).
    ///
    /// ¹ see the module-level comment for details
    pub fn render_camera(
        &self,
        handle: CameraHandle,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.cameras.get(handle).render(self, encoder, view);
    }

    /// Deletes camera, releasing its buffers.
    ///
    /// After this function is called, updating or rendering this camera will
    /// panic.
    pub fn delete_camera(&mut self, handle: CameraHandle) {
        self.cameras.remove(handle);
    }
}

/// Parameters used by Strolle to index textures, meshes etc.
///
/// This exists to faciliate integrations with existing systems, such as Bevy,
/// that already have their own newtypes for images, instances and so on.
pub trait Params
where
    Self: Clone + Debug,
{
    /// Handle used to lookup images.
    ///
    /// This corresponds to `Handle<Image>` in Bevy, but a simpler
    /// implementation can do with just `usize` or `String` as well.
    type ImageHandle: Eq + Hash + Clone + Debug;

    /// Image sampler; usually [`wgpu::Sampler`].
    ///
    /// This type parameter exists only because Bevy doesn't expose *owned*
    /// `wgpu::Sampler` directly, but rather through a newtype that derefs into
    /// the actual sampler.
    type ImageSampler: ImageSampler + Debug;

    /// Image texture; usually [`wgpu::TextureView`].
    ///
    /// Similarly as with samplers, this type parameter exists only because Bevy
    /// doesn't expose *owned* `wgpu::TextureView`.
    type ImageTexture: ImageTexture + Debug;

    /// Handle used to lookup instances of meshes.
    ///
    /// This corresponds to `Entity` in Bevy, but a simpler implementation can
    /// do with just `usize` or `String` as well.
    type InstanceHandle: Eq + Hash + Clone + Debug;

    /// Handle used to lookup lights.
    ///
    /// This corresponds to `Entity` in Bevy, but a simpler implementation can
    /// do with just `usize` or `String` as well.
    type LightHandle: Eq + Hash + Clone + Debug;

    /// Handle used to lookup materials.
    ///
    /// This corresponds to `Handle<StandardMaterial>` in Bevy, but a simpler
    /// implementation can do with just `usize` or `String` as well.
    type MaterialHandle: Eq + Hash + Clone + Debug;

    /// Handle used to lookup meshes.
    ///
    /// This corresponds to `Handle<Mesh>` in Bevy, but a simpler implementation
    /// can do with just `usize` or `String` as well.
    type MeshHandle: Eq + Hash + Clone + Debug;
}

#[derive(Debug)]
enum Event<P>
where
    P: Params,
{
    BufferReallocated,
    MaterialChanged(P::MaterialHandle),
    MaterialRemoved(P::MaterialHandle),
    ImageChanged(P::ImageHandle),
    ImageRemoved(P::ImageHandle),
}
