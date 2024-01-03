use bevy::asset::AssetId;
use bevy::render::mesh::Mesh as BevyMesh;

use crate::MeshTriangle;

#[derive(Clone, Debug)]
pub struct Mesh {
    triangles: Vec<MeshTriangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<MeshTriangle>) -> Self {
        Self { triangles }
    }

    pub(crate) fn triangles(&self) -> &[MeshTriangle] {
        &self.triangles
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshHandle(AssetId<BevyMesh>);

impl MeshHandle {
    pub fn new(asset: AssetId<BevyMesh>) -> Self {
        Self(asset)
    }
}
