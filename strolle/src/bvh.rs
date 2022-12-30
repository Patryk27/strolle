mod bounding_box;
mod builders;
mod bvh_node;
mod bvh_object;
mod bvh_printer;
mod bvh_serializer;
mod mesh_bvh;
mod world_bvh;

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use spirv_std::glam::Vec4;
use strolle_models::BvhPtr;

pub(crate) use self::bounding_box::*;
pub(self) use self::bvh_node::*;
pub(self) use self::bvh_object::*;
pub(self) use self::bvh_serializer::*;
pub use self::mesh_bvh::*;
pub use self::world_bvh::*;
use crate::buffers::StorageBufferable;

#[derive(Clone, Debug)]
pub struct Bvh<MeshHandle> {
    data: Vec<Vec4>,
    index: HashMap<MeshHandle, (BvhPtr, BvhPtr)>,
    got_dirty_meshes: bool,
}

impl<MeshHandle> Bvh<MeshHandle>
where
    MeshHandle: Eq + Hash + Debug,
{
    pub fn add_mesh(&mut self, mesh_handle: MeshHandle, mesh_bvh: MeshBvh) {
        assert!(!self.index.contains_key(&mesh_handle));

        let min_ptr = BvhPtr::new(self.data.len() as u32);
        BvhSerializer::process(&mut self.data, mesh_bvh.root());
        let max_ptr = BvhPtr::new((self.data.len() - 1) as u32);

        log::trace!(
            "BVH added: {:?} ({}..{})",
            mesh_handle,
            min_ptr.get(),
            max_ptr.get()
        );

        self.index.insert(mesh_handle, (min_ptr, max_ptr));
        self.got_dirty_meshes = true;
    }

    pub fn remove_mesh(&mut self, mesh_handle: &MeshHandle) {
        let Some((min_ptr, max_ptr)) = self.index.remove(mesh_handle) else { return };
        let len = max_ptr.get() - min_ptr.get() + 1;

        log::trace!(
            "BVH removed: {:?} ({}..{})",
            mesh_handle,
            min_ptr.get(),
            max_ptr.get()
        );

        self.data
            .drain((min_ptr.get() as usize)..=(max_ptr.get() as usize));

        for (min_ptr2, max_ptr2) in self.index.values_mut() {
            if min_ptr2.get() > min_ptr.get() {
                *min_ptr2.get_mut() -= len;
                *max_ptr2.get_mut() -= len;
            }
        }

        self.got_dirty_meshes = true;
    }

    pub fn get_mesh_metadata(
        &self,
        mesh_handle: &MeshHandle,
    ) -> Option<BvhPtr> {
        self.index.get(mesh_handle).map(|(min_id, _)| *min_id)
    }

    pub fn add_world(&mut self, world_bvh: WorldBvh) -> BvhPtr {
        // TODO cache this value
        let last_bvh_ptr = self
            .index
            .values()
            .map(|(_, max)| max.get())
            .max()
            .expect("World contains no meshes");

        self.data.truncate((last_bvh_ptr + 1) as usize);

        BvhSerializer::process(&mut self.data, world_bvh.root());
        BvhPtr::new(last_bvh_ptr + 1)
    }

    pub fn got_dirty_meshes(&self) -> bool {
        self.got_dirty_meshes
    }

    pub fn flush_dirty_meshes(&mut self) {
        self.got_dirty_meshes = false;
    }
}

impl<MeshHandle> Default for Bvh<MeshHandle> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            index: Default::default(),
            got_dirty_meshes: Default::default(),
        }
    }
}

impl<MeshHandle> StorageBufferable for Bvh<MeshHandle> {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
