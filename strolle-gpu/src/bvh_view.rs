use bytemuck::{Pod, Zeroable};
use glam::Vec4;

#[derive(Clone, Copy)]
pub struct BvhView<'a> {
    buffer: &'a [BvhNode],
}

impl<'a> BvhView<'a> {
    pub fn new(buffer: &'a [BvhNode]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, ptr: u32) -> BvhNode {
        unsafe { *self.buffer.get_unchecked(ptr as usize) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct BvhNode {
    pub d0: Vec4,
    pub d1: Vec4,
}

impl BvhNode {
    pub fn deserialize(&self) -> (bool, u32, u32) {
        let d0 = self.d0.x.to_bits();
        let d1 = self.d1.w.to_bits();

        let is_internal = d0 & 1 == 0;
        let arg0 = d0 >> 1;
        let arg1 = d1;

        (is_internal, arg0, arg1)
    }
}
