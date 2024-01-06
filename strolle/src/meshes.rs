use std::collections::HashMap;

use derivative::Derivative;

use crate::{Mesh, Params};

#[derive(Debug, Derivative)]
#[derivative(Default)]
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
    pub fn insert(&mut self, mesh_handle: P::MeshHandle, mesh: Mesh) {
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
