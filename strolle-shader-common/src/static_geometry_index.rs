use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StaticGeometryIndex {
    data: [Vec4; STATIC_GEOMETRY_INDEX_SIZE],
}

impl StaticGeometryIndex {
    pub fn read(&self, ptr: usize) -> Vec4 {
        unsafe { *self.data.get_unchecked(ptr) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl StaticGeometryIndex {
    pub fn new(data: [Vec4; STATIC_GEOMETRY_INDEX_SIZE]) -> Self {
        Self { data }
    }
}
