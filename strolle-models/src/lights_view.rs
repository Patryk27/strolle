use crate::{Light, LightId};

pub struct LightsView<'a> {
    data: &'a [Light],
}

impl<'a> LightsView<'a> {
    pub fn new(data: &'a [Light]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: LightId) -> Light {
        // TODO safety
        unsafe { *self.data.get_unchecked(id.get() as usize) }
    }
}
