use crate::utils::BoundingBox;
use crate::MeshTriangle;

#[derive(Clone, Debug)]
pub struct Mesh {
    triangles: Vec<MeshTriangle>,
    bounds: BoundingBox,
}

impl Mesh {
    pub fn new(triangles: Vec<MeshTriangle>, bounds: BoundingBox) -> Self {
        Self { triangles, bounds }
    }

    pub(crate) fn triangles(&self) -> &[MeshTriangle] {
        &self.triangles
    }

    pub(crate) fn bounds(&self) -> BoundingBox {
        self.bounds
    }
}
