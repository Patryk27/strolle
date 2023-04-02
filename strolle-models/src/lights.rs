use crate::{Light, LightId};

pub struct LightsView<'a> {
    items: &'a [Light],
}

impl<'a> LightsView<'a> {
    pub fn new(items: &'a [Light]) -> Self {
        Self { items }
    }

    pub fn get(&self, id: LightId) -> Light {
        unsafe { *self.items.get_unchecked(id.get() as usize) }
    }
}
