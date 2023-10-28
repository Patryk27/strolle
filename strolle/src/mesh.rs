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
