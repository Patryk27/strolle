use crate::{Triangle, TriangleId};

pub struct TrianglesView<'a> {
    data: &'a [Triangle],
}

impl<'a> TrianglesView<'a> {
    pub fn new(data: &'a [Triangle]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        // TODO safety
        unsafe { *self.data.get_unchecked(id.get() as usize) }
    }
}
