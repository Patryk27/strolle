use std::mem;

use crate::bvh::BvhNodeId;
use crate::utils::BoundingBox;
use crate::MeshTriangle;

#[derive(Clone, Debug)]
pub struct Mesh {
    triangles: Vec<MeshTriangle>,
    bounds: BoundingBox,
    node_id: Option<BvhNodeId>,
}

impl Mesh {
    pub fn new(triangles: Vec<MeshTriangle>, bounds: BoundingBox) -> Self {
        Self {
            triangles,
            bounds,
            node_id: None,
        }
    }

    pub(crate) fn take_triangles(&mut self) -> Vec<MeshTriangle> {
        mem::take(&mut self.triangles)
    }

    pub(crate) fn bounds(&self) -> BoundingBox {
        self.bounds
    }

    pub(crate) fn node_id(&self) -> Option<BvhNodeId> {
        self.node_id
    }

    pub(crate) fn node_id_mut(&mut self) -> &mut Option<BvhNodeId> {
        &mut self.node_id
    }
}
