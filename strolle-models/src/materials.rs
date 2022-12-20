use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Materials {
    items: [Material; MAX_MATERIALS as _],
}

impl Materials {
    pub fn get(&self, id: MaterialId) -> Material {
        unsafe { *self.items.get_unchecked(id.get()) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Materials {
    pub fn set(&mut self, id: MaterialId, item: Material) {
        self.items[id.get()] = item;
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for Materials {
    fn default() -> Self {
        Self::zeroed()
    }
}
