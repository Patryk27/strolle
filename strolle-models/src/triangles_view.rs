use crate::{Triangle, TriangleId};

pub struct TrianglesView<'a> {
    items: &'a [Triangle],
}

impl<'a> TrianglesView<'a> {
    pub fn new(items: &'a [Triangle]) -> Self {
        Self { items }
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        unsafe { *self.items.get_unchecked(id.get() as usize) }
    }
}
