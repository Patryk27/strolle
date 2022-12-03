use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StaticGeometry {
    items: [Triangle; MAX_STATIC_TRIANGLES],
}

impl StaticGeometry {
    pub fn get(&self, id: TriangleId<StaticTriangle>) -> Triangle {
        unsafe { *self.items.get_unchecked(id.get()) }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl StaticGeometry {
    pub fn set(&mut self, id: TriangleId<StaticTriangle>, item: Triangle) {
        self.items[id.get()] = item;
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (TriangleId<StaticTriangle>, Triangle)> + '_ {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, triangle)| triangle.is_some())
            .map(|(id, triangle)| (TriangleId::new_static(id), *triangle))
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for StaticGeometry {
    fn default() -> Self {
        Self::zeroed()
    }
}
