use glam::Vec4;
use spirv_std::arch::IndexUnchecked;

#[derive(Clone, Copy)]
pub struct BvhView<'a> {
    buffer: &'a [Vec4],
}

impl<'a> BvhView<'a> {
    pub fn new(buffer: &'a [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, ptr: u32) -> Vec4 {
        unsafe { *self.buffer.index_unchecked(ptr as usize) }
    }
}
