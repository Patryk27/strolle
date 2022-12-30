use crate::{Material, MaterialId};

pub struct MaterialsView<'a> {
    data: &'a [Material],
}

impl<'a> MaterialsView<'a> {
    pub fn new(data: &'a [Material]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: MaterialId) -> Material {
        // TODO safety
        unsafe { *self.data.get_unchecked(id.get() as usize) }
    }
}
