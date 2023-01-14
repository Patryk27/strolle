use crate::RayOp;

pub struct RayOpsView<'a> {
    items: &'a mut [RayOp],
}

impl<'a> RayOpsView<'a> {
    pub fn new(items: &'a mut [RayOp]) -> Self {
        Self { items }
    }

    pub fn get(&self, idx: u32) -> RayOp {
        unsafe { *self.items.get_unchecked(idx as usize) }
    }

    pub fn set(&mut self, idx: u32, ray: RayOp) {
        unsafe {
            *self.items.get_unchecked_mut(idx as usize) = ray;
        }
    }
}
