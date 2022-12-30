#![feature(type_alias_impl_trait)]

mod buffers;
mod bvh;
mod instances;
mod lights;
mod materials;
mod triangles;
mod viewport;

use std::fmt::Debug;
use std::hash::Hash;
use std::mem;
use std::time::Instant;

use spirv_std::glam::{Mat4, UVec2, Vec4};
pub use strolle_models::*;

pub(crate) use self::buffers::*;
pub(crate) use self::bvh::*;
pub(crate) use self::instances::*;
pub(crate) use self::lights::*;
pub(crate) use self::materials::*;
pub(crate) use self::triangles::*;
pub use self::viewport::*;

pub struct Engine<Params>
where
    Params: EngineParams,
{
    tracer: wgpu::ShaderModule,
    materializer: wgpu::ShaderModule,
    printer: wgpu::ShaderModule,
    triangles: StorageBuffer<Triangles<Params::MeshHandle>>,
    instances: StorageBuffer<Instances>,
    bvh: StorageBuffer<Bvh<Params::MeshHandle>>,
    lights: StorageBuffer<Lights<Params::LightHandle>>,
    materials: StorageBuffer<Materials<Params::MaterialHandle>>,
    info: UniformBuffer<Info>,
}

impl<P> Engine<P>
where
    P: EngineParams,
{
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

        let triangles =
            StorageBuffer::new_default(device, "strolle_triangles", BUF_SIZE);

        let instances =
            StorageBuffer::new_default(device, "strolle_instances", BUF_SIZE);

        let bvh =
            StorageBuffer::new_default(device, "strolle_bvh", 2 * BUF_SIZE);

        let lights = StorageBuffer::new_default(
            device,
            "strolle_lights",
            4 * 1024 * 1024,
        );

        let materials = StorageBuffer::new_default(
            device,
            "strolle_materials",
            4 * 1024 * 1024,
        );

        let info = UniformBuffer::new_default(device, "strolle_info");

        Self {
            tracer,
            materializer,
            printer,
            triangles,
            instances,
            bvh,
            lights,
            materials,
            info,
        }
    }

    pub fn add_mesh(
        &mut self,
        mesh_handle: P::MeshHandle,
        mesh_tris: Vec<Triangle>,
    ) {
        self.bvh
            .add_mesh(mesh_handle.clone(), MeshBvh::build(&mesh_tris));

        self.triangles.add(mesh_handle, mesh_tris);
    }

    pub fn remove_mesh(&mut self, mesh_handle: &P::MeshHandle) {
        self.bvh.remove_mesh(mesh_handle);
        self.triangles.remove(mesh_handle);
    }

    pub fn contains_mesh(&self, mesh_handle: &P::MeshHandle) -> bool {
        self.bvh.get_mesh_metadata(mesh_handle).is_some()
    }

    pub fn add_material(
        &mut self,
        material_handle: P::MaterialHandle,
        material: Material,
    ) {
        self.materials.add(material_handle, material);
    }

    pub fn remove_material(&mut self, material_handle: &P::MaterialHandle) {
        self.materials.remove(material_handle);
    }

    pub fn add_instance(
        &mut self,
        mesh_handle: P::MeshHandle,
        material_handle: P::MaterialHandle,
        transform: Mat4,
    ) {
        assert!(
            transform.row(3).abs_diff_eq(Vec4::W, 1e-6),
            "Instances with perspetive-projection transforms are not supported"
        );

        // TODO this assumes `.clear_instances()` is called each frame
        let (min_triangle_id, max_triangle_id, bounding_box) = self
            .triangles
            .try_get_metadata(&mesh_handle)
            .unwrap_or_else(|| {
                panic!("Mesh not known: {:?}", mesh_handle);
            });

        let bounding_box = bounding_box.transform(transform);

        let material_id =
            self.materials.lookup(&material_handle).unwrap_or_else(|| {
                panic!("Material not known: {:?}", material_handle);
            });

        // TODO this assumes `.clear_instances()` is called each frame
        let bvh_ptr =
            self.bvh.get_mesh_metadata(&mesh_handle).unwrap_or_else(|| {
                panic!("Mesh not known: {:?}", mesh_handle);
            });

        let instance = Instance::new(
            transform,
            min_triangle_id,
            max_triangle_id,
            material_id,
            bvh_ptr,
        );

        self.instances.add(instance, bounding_box);
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
    }

    pub fn add_light(&mut self, light_handle: P::LightHandle, light: Light) {
        self.lights.add(light_handle, light);
    }

    pub fn remove_light(&mut self, light_handle: &P::LightHandle) {
        self.lights.remove(light_handle);
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    pub fn write(&mut self, queue: &wgpu::Queue) {
        let tt = Instant::now();

        // If the scene contains no instances, let's just zero-out world's
        // bvh-ptr (which otherwise cannot be zero, so it serves as a cheap
        // sentinel value), flush the info-buffer and bail out.
        //
        // We don't have to flush any other buffers, because our shaders won't
        // access them anyway - the first thing we do in shaders is checking
        // this bvh-pointer and insta-returning if it's zero.
        if self.instances.is_empty() {
            self.info.world_bvh_ptr = 0;
            self.info.flush(queue);

            return;
        }

        // ---------------------- //
        // Phase 1: Flush buffers //

        self.info.light_count = self.lights.len();
        self.info.flush(queue);
        self.triangles.flush(queue);
        self.instances.flush(queue);
        self.lights.flush(queue);
        self.materials.flush(queue);

        // ------------------------------------ //
        // Phase 2: Generate BVH (and flush it) //

        // In principle, we have to rebuild world-bvh only if some of the meshes
        // or instances have changed (were moved, scaled etc.); in practice we
        // are doing it every frame anyway, since it allows us to benchmark the
        // algorithm and make sure it's real-time
        let world_bvh = WorldBvh::build(&self.instances);
        let world_bvh_ptr = self.bvh.add_world(world_bvh);

        // Now, as for the flushing:
        //
        // The easiest thing would be to send the entire buffer every time, but
        // for bigger scenes we're talking about hundreds of megabytes sent to
        // VRAM each frame - that's a lot!
        //
        // (for the Nefertiti model sending the entire BVH takes around 5 ms.)
        //
        // So, what we do instead is try to be lazy:
        //
        // - (the rare case) if some meshes have changed (i.e. we've got new
        //   meshes, updated meshes or deleted meshes), we flush the entire BVH,
        //
        // - (the typical case) if only world-bvh has changed, we send only it,
        //   since VRAM will have already contained mesh-bvhs; that's also the
        //   reason why we keep world-bvh at the very end of the buffer, since
        //   otherwise this trick wouldn't work
        if self.bvh.got_dirty_meshes() || self.info.world_bvh_ptr == 0 {
            // The Rare Case (TM)

            self.info.world_bvh_ptr = world_bvh_ptr.get();
            self.bvh.flush(queue);
            self.bvh.flush_dirty_meshes();
        } else {
            // The Typical Case (TM)

            assert_eq!(self.info.world_bvh_ptr, world_bvh_ptr.get());

            let offset =
                (world_bvh_ptr.get() as usize) * mem::size_of::<Vec4>();

            self.bvh.flush_offset(queue, offset);
        }

        log::trace!("write() took {:?}", tt.elapsed());
    }

    pub fn create_viewport(
        &self,
        device: &wgpu::Device,
        pos: UVec2,
        size: UVec2,
        format: wgpu::TextureFormat,
        camera: Camera,
    ) -> Viewport {
        Viewport::new(self, device, pos, size, format, camera)
    }
}

pub trait EngineParams {
    type MeshHandle: Eq + Hash + Clone + Debug;
    type LightHandle: Eq + Hash + Clone + Debug;
    type MaterialHandle: Eq + Hash + Clone + Debug;
}
