use crate::{Triangle, TriangleId};

#[derive(Clone, Copy)]
pub struct TrianglesView<'a> {
    buffer: &'a [Triangle],
}

impl<'a> TrianglesView<'a> {
    pub fn new(buffer: &'a [Triangle]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        unsafe { *self.buffer.get_unchecked(id.get() as usize) }
    }
}
