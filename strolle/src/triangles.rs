use std::collections::HashMap;
use std::fmt::Debug;

use strolle_models as gpu;

use crate::bvh::BoundingBox;
use crate::{Params, StorageBufferable};

#[derive(Clone, Debug)]
pub struct Triangles<P>
where
    P: Params,
{
    gpu_triangles: Vec<gpu::Triangle>,
    index:
        HashMap<P::MeshHandle, (gpu::TriangleId, gpu::TriangleId, BoundingBox)>,
}

impl<P> Triangles<P>
where
    P: Params,
{
    pub fn add(
        &mut self,
        mesh_handle: P::MeshHandle,
        mesh_triangles: Vec<gpu::Triangle>,
    ) {
        assert!(!self.index.contains_key(&mesh_handle));

        let bounding_box = BoundingBox::from_points(
            mesh_triangles
                .iter()
                .flat_map(|triangle| triangle.vertices()),
        );

        let min_id = gpu::TriangleId::new(self.gpu_triangles.len() as u32);

        self.gpu_triangles.extend(mesh_triangles);

        let max_id =
            gpu::TriangleId::new((self.gpu_triangles.len() - 1) as u32);

        log::debug!(
            "Triangles added: {:?} ({}..{})",
            mesh_handle,
            min_id.get(),
            max_id.get()
        );

        self.index
            .insert(mesh_handle, (min_id, max_id, bounding_box));
    }

    pub fn remove(&mut self, mesh_handle: &P::MeshHandle) {
        let Some((min_id, max_id, _)) = self.index.remove(mesh_handle) else { return };
        let len = max_id.get() - min_id.get() + 1;

        log::debug!(
            "Triangles removed: {:?} ({}..{})",
            mesh_handle,
            min_id.get(),
            max_id.get()
        );

        self.gpu_triangles
            .drain((min_id.get() as usize)..=(max_id.get() as usize));

        for (min_id2, max_id2, _) in self.index.values_mut() {
            if min_id2.get() > min_id.get() {
                *min_id2.get_mut() -= len;
                *max_id2.get_mut() -= len;
            }
        }
    }

    pub fn lookup(
        &self,
        mesh_handle: &P::MeshHandle,
    ) -> Option<(gpu::TriangleId, gpu::TriangleId, BoundingBox)> {
        self.index.get(mesh_handle).copied()
    }
}

impl<P> Default for Triangles<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            gpu_triangles: Default::default(),
            index: Default::default(),
        }
    }
}

impl<P> StorageBufferable for Triangles<P>
where
    P: Params,
{
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.gpu_triangles)
    }
}
