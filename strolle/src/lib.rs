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
//! Material determines how an object should look like, i.e. whether it should
//! be transparent etc.
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
//! Light defines how the scene should get lightnened - i.e. whether it's a
//! point-lightning, a cone-lightning etc.

#![feature(hash_raw_entry)]
#![feature(lint_reasons)]

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
mod primitive;
mod primitives;
mod shaders;
mod sun;
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
pub(crate) use self::primitive::*;
pub(crate) use self::primitives::*;
pub(crate) use self::shaders::*;
pub use self::sun::*;
pub(crate) use self::triangles::*;
pub use self::utils::BoundingBox;
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
    primitives: Primitives<P>,
    bvh: Bvh,
    lights: Lights<P>,
    images: Images<P>,
    materials: Materials<P>,
    world: MappedUniformBuffer<gpu::World>,
    cameras: CameraControllers,
    sun: Sun,
    frame: u32,
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
            primitives: Primitives::default(),
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
            frame: 0,
            has_dirty_materials: false,
            has_dirty_images: false,
            has_dirty_sun: true,
            print_stats: env::var("STROLLE_STATS").as_deref() == Ok("1"),
        }
    }

    /// Creates a mesh¹ or updates the existing one.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_mesh(&mut self, mesh_handle: P::MeshHandle, mut mesh: Mesh) {
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
        self.materials.add(material_handle, material);
        self.has_dirty_materials = true;
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
        self.has_dirty_materials = true;
    }

    /// Creates an image¹ or updates the existing one.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_image(&mut self, image_handle: P::ImageHandle, image: Image<P>) {
        self.images.add(image_handle, image);
        self.has_dirty_images = true;
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
        self.has_dirty_images = true;
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
        if let Some(instance) = self.instances.remove(instance_handle) {
            self.triangles.remove(instance_handle);
            self.primitives.tlas_mut().remove(instance_handle);

            if instance.inline {
                self.primitives.delete_blas(*instance_handle);
            }
        }
    }

    /// Creates a light or updates the existing one¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn add_light(&mut self, light_handle: P::LightHandle, light: Light) {
        self.lights.add(light_handle, light);
    }

    /// Removes a light¹.
    ///
    /// ¹ see the module-level comment for details
    pub fn remove_light(&mut self, light_handle: &P::LightHandle) {
        self.lights.remove(light_handle);
    }

    /// Updates sun's parameters.
    pub fn update_sun(&mut self, sun: Sun) {
        self.sun = sun;
        self.has_dirty_sun = true;
    }

    /// Sends all changes to the GPU and prepares it for the upcoming frame.
    ///
    /// This function must be called before each frame, i.e. before invoking
    /// [`Self::render_camera()`].
    ///
    /// (if you have multiple cameras, calling this function just once is
    /// enough.)
    pub fn tick(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let tt = Instant::now();
        let any_material_modified = mem::take(&mut self.has_dirty_materials);
        let any_image_modified = mem::take(&mut self.has_dirty_images);

        utils::measure("tick.noise", || {
            self.noise.flush(device, queue);
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
                &mut self.meshes,
                &self.materials,
                &mut self.triangles,
                &mut self.primitives,
                &mut self.bvh,
            )
        });

        if any_instance_changed {
            utils::measure("tick.bvh", || {
                self.bvh.refresh(
                    &self.materials,
                    &self.instances,
                    &mut self.primitives,
                );
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

        self.frame += 1;

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

// TODO get rid of
pub trait Params
where
    Self: Clone + Debug + PartialEq + Eq + Hash + Send + Sync + Default,
{
    type ImageHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync;
    type ImageTexture: Debug + Deref<Target = wgpu::Texture>;
    type InstanceHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync;
    type LightHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync;
    type MaterialHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync;
    type MeshHandle: Clone + Copy + Eq + Hash + Debug + Send + Sync;
}
