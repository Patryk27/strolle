use spirv_std::arch::IndexUnchecked;

use crate::{Light, LightId};

#[derive(Clone, Copy)]
pub struct LightsView<'a> {
    items: &'a [Light],
}

impl<'a> LightsView<'a> {
    pub fn new(items: &'a [Light]) -> Self {
        Self { items }
    }

    pub fn get(&self, id: LightId) -> Light {
        unsafe { *self.items.index_unchecked(id.get() as usize) }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
