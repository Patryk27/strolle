use std::collections::HashMap;

use crate::{Mesh, Params};

#[derive(Debug)]
pub struct Meshes<P>
where
    P: Params,
{
    meshes: HashMap<P::MeshHandle, Mesh>,
}

impl<P> Meshes<P>
where
    P: Params,
{
    pub fn add(&mut self, mesh_handle: P::MeshHandle, mesh: Mesh) {
        self.meshes.insert(mesh_handle, mesh);
    }

    pub fn get(&self, mesh_handle: &P::MeshHandle) -> Option<&Mesh> {
        self.meshes.get(mesh_handle)
    }

    pub fn remove(&mut self, mesh_handle: &P::MeshHandle) {
        self.meshes.remove(mesh_handle);
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }
}

impl<P> Default for Meshes<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            meshes: Default::default(),
        }
    }
}
