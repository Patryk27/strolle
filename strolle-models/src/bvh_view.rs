use glam::Vec4;

#[derive(Clone, Copy)]
pub struct BvhView<'a> {
    items: &'a [Vec4],
}

impl<'a> BvhView<'a> {
    pub fn new(items: &'a [Vec4]) -> Self {
        Self { items }
    }

    pub fn get(&self, ptr: u32) -> Vec4 {
        unsafe { *self.items.get_unchecked(ptr as usize) }
    }
}
