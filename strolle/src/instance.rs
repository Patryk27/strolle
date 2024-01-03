use bevy::ecs::entity::Entity;
use glam::Affine3A;

use crate::{MaterialHandle, MeshHandle};

#[derive(Debug)]
pub struct Instance {
    pub(crate) mesh_handle: MeshHandle,
    pub(crate) material_handle: MaterialHandle,
    pub(crate) transform: Affine3A,
    pub(crate) transform_inverse: Affine3A,
}

impl Instance {
    pub fn new(
        mesh_handle: MeshHandle,
        material_handle: MaterialHandle,
        transform: Affine3A,
    ) -> Self {
        Self {
            mesh_handle,
            material_handle,
            transform,
            transform_inverse: transform.inverse(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceHandle(Entity);

impl InstanceHandle {
    pub fn new(asset: Entity) -> Self {
        Self(asset)
    }
}
