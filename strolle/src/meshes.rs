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
    pub fn insert(&mut self, handle: P::MeshHandle, item: Mesh) {
        self.meshes.insert(handle, item);
    }

    pub fn get(&self, handle: P::MeshHandle) -> Option<&Mesh> {
        self.meshes.get(&handle)
    }

    pub fn remove(&mut self, handle: P::MeshHandle) {
        self.meshes.remove(&handle);
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }
}
