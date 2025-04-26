//! Strolle, an experimental real-time renderer that supports global
//! illumination.
//!
//! # Usage
//!
//! If you're using Bevy, please take a look at the `bevy-strolle` crate that
//! provides an integration with Bevy.
//!
//! It's also possible to uses Strolle outside of Bevy, as the low-level
//! interface requires only `wgpu` - there is no tutorial for that just yet,
//! though.
//!
//! # Definitions
//!
//! ## Mesh
//!
//! Mesh defines the structure of an object; it contains triangles, but without
//! any information about the materials.
//!
//! Meshes together with materials create instances.
//!
//! ## Material
//!
//! Material determines how an object should look like - its color, whether it
//! should be transparent or not etc.
//!
//! Materials together with meshes create instances.
//!
//! ## Image
//!
//! Image can be used to enhance material's properties, e.g. to make its diffuse
//! color more interesting.
//!
//! Note that normal maps are also classified as images.
//!
//! ## Instance
//!
//! Instance defines a single object as visible in the world-space; mesh +
//! material + transformation matrix create a single instance.
//!
//! ## Light
//!
//! Light defines how the scene should get lightened - i.e. whether it's a
//! point-light, a cone-light etc.

#![feature(hash_raw_entry)]

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
mod mesh_triangle;
mod meshes;
mod noise;
mod shaders;
mod sun;
mod triangle;
mod triangles;
mod utils;

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::time::Instant;
use std::{env, mem};

pub use glam;
use log::{info, trace};
use strolle_gpu as gpu;

pub(crate) use self::buffers::*;
pub(crate) use self::bvh::*;
pub use self::camera::*;
pub(crate) use self::camera_controller::*;
pub(crate) use self::camera_controllers::*;
pub use self::image::*;
pub(crate) use self::images::*;
pub use self::instance::*;
pub(crate) use self::instances::*;
pub use self::light::*;
pub(crate) use self::lights::*;
pub use self::material::*;
pub(crate) use self::materials::*;
pub use self::mesh::*;
pub use self::mesh_triangle::*;
pub(crate) use self::meshes::*;
pub(crate) use self::noise::*;
pub(crate) use self::shaders::*;
pub use self::sun::*;
pub(crate) use self::triangle::*;
pub(crate) use self::triangles::*;
pub(crate) use self::utils::*;

#[derive(Debug)]
pub struct Engine<P>
where
    P: Params,
{
    shaders: Shaders,
    noise: Noise,
    meshes: Meshes<P>,
    instances: Instances<P>,
    triangles: Triangles<P>,
    bvh: Bvh,
    lights: Lights<P>,
    images: Images<P>,
    materials: Materials<P>,
    world: MappedUniformBuffer<gpu::World>,
    cameras: CameraControllers,
    sun: Sun,
    frame: gpu::Frame,
    has_dirty_materials: bool,
    has_dirty_images: bool,
    has_dirty_sun: bool,
    print_stats: bool,
}

impl<P> Engine<P>
where
    P: Params,
{
    pub fn new(device: &wgpu::Device) -> Self {
        info!("Initializing");

        Self {
            shaders: Shaders::new(device),
            noise: Noise::new(device),
            meshes: Meshes::default(),
            instances: Instances::default(),
            triangles: Triangles::new(device),
            bvh: Bvh::new(device),
            lights: Lights::new(device),
            images: Images::new(device),
            materials: Materials::new(device),
            world: MappedUniformBuffer::new(
                device,
                "world",
                Default::default(),
            ),
            cameras: Default::default(),
            sun: Default::default(),
            frame: gpu::Frame::new(1),
            has_dirty_materials: false,
            has_dirty_images: false,
            has_dirty_sun: true,
            print_stats: env::var("STROLLE_STATS").as_deref() == Ok("1"),
        }
    }

    /// Creates or updates a mesh.
    pub fn insert_mesh(&mut self, handle: P::MeshHandle, item: Mesh) {
        self.meshes.insert(handle, item);
    }

    /// Removes a mesh.
    ///
    /// Note that removing a mesh doesn't automatically remove instances that
    /// refer to this mesh.
    pub fn remove_mesh(&mut self, handle: P::MeshHandle) {
        self.meshes.remove(handle);
    }

    /// Creates or updates a material.
    pub fn insert_material(
        &mut self,
        handle: P::MaterialHandle,
        item: Material<P>,
    ) {
        self.materials.insert(handle, item);
        self.has_dirty_materials = true;
    }

    /// Returns whether given material exists.
    pub fn has_material(&self, handle: P::MaterialHandle) -> bool {
        self.materials.has(handle)
    }

    /// Removes a material.
    ///
    /// Note that removing a material doesn't automatically remove instances
    /// that refer to this material.
    pub fn remove_material(&mut self, handle: P::MaterialHandle) {
        self.materials.remove(handle);
        self.has_dirty_materials = true;
    }

    /// Creates or updates an image.
    pub fn insert_image(
        &mut self,
        image_handle: P::ImageHandle,
        image: Image<P>,
    ) {
        self.images.insert(image_handle, image);
        self.has_dirty_images = true;
    }

    /// Removes an image.
    ///
    /// Note that removing an image doesn't automatically remove materials that
    /// refer to this image.
    pub fn remove_image(&mut self, handle: P::ImageHandle) {
        self.images.remove(handle);
        self.has_dirty_images = true;
    }

    /// Creates or updates an instance.
    pub fn insert_instance(
        &mut self,
        instance_handle: P::InstanceHandle,
        instance: Instance<P>,
    ) {
        self.instances.insert(instance_handle, instance);
    }

    /// Removes an instance.
    pub fn remove_instance(&mut self, handle: P::InstanceHandle) {
        self.instances.remove(handle);
        self.triangles.remove(&mut self.bvh, handle);
    }

    /// Creates or updates a light.
    pub fn insert_light(&mut self, handle: P::LightHandle, item: Light) {
        self.lights.insert(handle, item);
    }

    /// Removes a light.
    pub fn remove_light(&mut self, handle: P::LightHandle) {
        self.lights.remove(handle);
    }

    /// Updates sun's parameters.
    pub fn update_sun(&mut self, sun: Sun) {
        self.sun = sun;
        self.has_dirty_sun = true;
    }

    /// Creates a new camera that can be used to render the world.
    ///
    /// Note that this is a pretty heavy operation that allocates per-camera
    /// buffers etc., and so it's expected that you only call this function when
    /// necessary (not, say, each frame).
    pub fn create_camera(
        &mut self,
        device: &wgpu::Device,
        camera: Camera,
    ) -> CameraHandle {
        self.cameras
            .add(CameraController::new(self, device, camera))
    }

    /// Updates camera, changing its mode, position, size etc.
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

    /// Renders camera to texture.
    ///
    /// Note that `view`'s texture format must be the same as the format given
    /// to [`Self::create_camera()`].
    pub fn render_camera(
        &self,
        handle: CameraHandle,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.cameras.get(handle).render(self, encoder, view);
    }

    /// Deletes a camera.
    ///
    /// After this function is called, updating or rendering this camera will
    /// panic.
    pub fn delete_camera(&mut self, handle: CameraHandle) {
        self.cameras.remove(handle);
    }

    /// Sends all changes to the GPU and prepares it for the upcoming frame.
    ///
    /// This function must be called before invoking [`Self::render_camera()`]
    /// (if you have multiple cameras, calling this function just once is
    /// enough.)
    pub fn tick(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let tt = Instant::now();
        let any_material_modified = mem::take(&mut self.has_dirty_materials);
        let any_image_modified = mem::take(&mut self.has_dirty_images);

        utils::measure("tick.noise", || {
            self.noise.flush(queue);
        });

        utils::measure("tick.images", || {
            self.images.flush(device, queue);
        });

        if any_material_modified || any_image_modified {
            utils::measure("tick.materials", || {
                self.materials.refresh(&self.images);
            });
        }

        // ---

        let any_instance_changed = utils::measure("tick.instances", || {
            self.instances.refresh(
                &self.meshes,
                &self.materials,
                &mut self.triangles,
                &mut self.bvh,
            )
        });

        if any_instance_changed {
            utils::measure("tick.bvh", || {
                self.bvh.refresh(&self.materials);
            });
        }

        // ---

        *self.world = gpu::World {
            light_count: self.lights.len(),
            sun_azimuth: self.sun.azimuth,
            sun_altitude: self.sun.altitude,
        };

        utils::measure("tick.world", || {
            self.world.flush(queue);
        });

        if mem::take(&mut self.has_dirty_sun) {
            self.lights.update_sun(*self.world);
        }

        let any_buffer_reallocated = utils::measure("tick.buffers", || {
            false
                | self.bvh.flush(device, queue).reallocated
                | self.triangles.flush(device, queue).reallocated
                | self.lights.flush(device, queue).reallocated
                | self.materials.flush(device, queue).reallocated
        });

        // ---

        if any_buffer_reallocated {
            let mut cameras = mem::take(&mut self.cameras);

            for camera in cameras.iter_mut() {
                camera.invalidate(self, device);
            }

            self.cameras = cameras;
        }

        utils::measure("tick.cameras", || {
            for camera in self.cameras.iter_mut() {
                camera.flush(self.frame, queue);
            }
        });

        self.frame = gpu::Frame::new(self.frame.get() + 1);

        // ---

        utils::metric("tick", tt);

        if self.print_stats {
            trace!(
                "meshes={} | triangles={} | nodes={} | materials={} | lights = {}",
                self.meshes.len(),
                self.triangles.len(),
                self.bvh.len(),
                self.materials.len(),
                self.lights.len(),
            );
        }
    }
}

/// Parameters used by Strolle to index textures, meshes etc.
///
/// This exists to faciliate integrations with existing systems, such as Bevy,
/// that already have their own newtypes for images, instances and so on.
pub trait Params {
    type ImageHandle: Clone + Copy + Debug + Eq + Hash;
    type ImageTexture: Debug + Deref<Target = wgpu::Texture>;
    type InstanceHandle: Clone + Copy + Debug + Eq + Hash;
    type LightHandle: Clone + Copy + Debug + Eq + Hash;
    type MaterialHandle: Clone + Copy + Debug + Eq + Hash;
    type MeshHandle: Clone + Copy + Debug + Eq + Hash;
}
