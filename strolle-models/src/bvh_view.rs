use glam::Vec4;

pub struct BvhView<'a> {
    data: &'a [Vec4],
}

impl<'a> BvhView<'a> {
    pub fn new(data: &'a [Vec4]) -> Self {
        Self { data }
    }

    pub fn get(&self, ptr: u32) -> Vec4 {
        // TODO safety
        unsafe { *self.data.get_unchecked(ptr as usize) }
    }
}
