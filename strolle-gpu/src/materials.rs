use spirv_std::arch::IndexUnchecked;

use crate::{Material, MaterialId};

#[derive(Clone, Copy)]
pub struct MaterialsView<'a> {
    items: &'a [Material],
}

impl<'a> MaterialsView<'a> {
    pub fn new(items: &'a [Material]) -> Self {
        Self { items }
    }

    pub fn get(self, id: MaterialId) -> Material {
        unsafe { *self.items.index_unchecked(id.get() as usize) }
    }
}
