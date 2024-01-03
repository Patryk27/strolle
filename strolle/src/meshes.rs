use std::collections::HashMap;

use bevy::ecs::system::Resource;

use crate::{Mesh, MeshHandle};

#[derive(Debug, Default, Resource)]
pub struct Meshes {
    meshes: HashMap<MeshHandle, Mesh>,
}

impl Meshes {
    pub fn add(&mut self, handle: MeshHandle, mesh: Mesh) {
        self.meshes.insert(handle, mesh);
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(&handle)
    }

    pub fn remove(&mut self, handle: MeshHandle) {
        self.meshes.remove(&handle);
    }

    pub fn len(&self) -> usize {
        self.meshes.len()
    }
}
