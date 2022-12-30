use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use strolle_models::{Triangle, TriangleId};

use crate::bvh::BoundingBox;
use crate::StorageBufferable;

#[derive(Clone, Debug)]
pub struct Triangles<MeshHandle> {
    data: Vec<Triangle>,
    index: HashMap<MeshHandle, (TriangleId, TriangleId, BoundingBox)>,
}

impl<MeshHandle> Triangles<MeshHandle>
where
    MeshHandle: Eq + Hash + Debug,
{
    pub fn add(
        &mut self,
        mesh_handle: MeshHandle,
        mesh_triangles: Vec<Triangle>,
    ) {
        assert!(!self.index.contains_key(&mesh_handle));

        let bounding_box = BoundingBox::from_points(
            mesh_triangles
                .iter()
                .flat_map(|triangle| triangle.vertices()),
        );

        let min_id = TriangleId::new(self.data.len() as u32);
        self.data.extend(mesh_triangles);
        let max_id = TriangleId::new((self.data.len() - 1) as u32);

        log::trace!(
            "Triangles added: {:?} ({}..{})",
            mesh_handle,
            min_id.get(),
            max_id.get()
        );

        self.index
            .insert(mesh_handle, (min_id, max_id, bounding_box));
    }

    pub fn remove(&mut self, mesh_handle: &MeshHandle) {
        let Some((min_id, max_id, _)) = self.index.remove(mesh_handle) else { return };
        let len = max_id.get() - min_id.get() + 1;

        log::trace!(
            "Triangles removed: {:?} ({}..{})",
            mesh_handle,
            min_id.get(),
            max_id.get()
        );

        self.data
            .drain((min_id.get() as usize)..=(max_id.get() as usize));

        for (min_id2, max_id2, _) in self.index.values_mut() {
            if min_id2.get() > min_id.get() {
                *min_id2.get_mut() -= len;
                *max_id2.get_mut() -= len;
            }
        }
    }

    pub fn try_get_metadata(
        &self,
        mesh_handle: &MeshHandle,
    ) -> Option<(TriangleId, TriangleId, BoundingBox)> {
        self.index.get(mesh_handle).copied()
    }
}

impl<MeshHandle> Default for Triangles<MeshHandle> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            index: Default::default(),
        }
    }
}

impl<MeshHandle> StorageBufferable for Triangles<MeshHandle> {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
