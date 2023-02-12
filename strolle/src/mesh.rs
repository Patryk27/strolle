use crate::Triangle;

#[derive(Clone, Debug)]
pub struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }

    pub(crate) fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }
}
