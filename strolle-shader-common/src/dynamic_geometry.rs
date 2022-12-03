use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DynamicGeometry {
    items: [Triangle; MAX_DYNAMIC_TRIANGLES],
    len: PadU32,
}

impl DynamicGeometry {
    pub fn get(&self, id: TriangleId<DynamicTriangle>) -> Triangle {
        unsafe { *self.items.get_unchecked(id.get()) }
    }

    pub fn len(&self) -> usize {
        self.len.value as _
    }
}

#[cfg(not(target_arch = "spirv"))]
impl DynamicGeometry {
    pub fn get_mut(
        &mut self,
        id: TriangleId<DynamicTriangle>,
    ) -> &mut Triangle {
        &mut self.items[id.get()]
    }

    pub fn push(
        &mut self,
        item: Triangle,
    ) -> Option<TriangleId<DynamicTriangle>> {
        if self.len() == MAX_DYNAMIC_TRIANGLES {
            return None;
        }

        let id = TriangleId::new_dynamic(self.len());

        self.items[id.get()] = item;
        self.len += 1;

        Some(id)
    }

    pub fn set(&mut self, id: TriangleId<DynamicTriangle>, item: Triangle) {
        self.items[id.get()] = item;
    }

    pub fn remove(&mut self, id: TriangleId<DynamicTriangle>) {
        assert!(id.get() < self.len.value as usize);

        for id in id.get()..(MAX_DYNAMIC_TRIANGLES - 1) {
            self.items[id] = self.items[id + 1];
        }

        self.len -= 1;
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for DynamicGeometry {
    fn default() -> Self {
        Self::zeroed()
    }
}
