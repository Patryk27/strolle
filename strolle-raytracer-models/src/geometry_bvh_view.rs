use crate::*;

pub struct GeometryBvhView<'a> {
    data: &'a [Vec4],
}

impl<'a> GeometryBvhView<'a> {
    pub fn new(data: &'a [Vec4]) -> Self {
        Self { data }
    }

    pub fn read(&self, ptr: usize) -> Vec4 {
        // TODO safety
        unsafe { *self.data.get_unchecked(ptr) }
    }
}
