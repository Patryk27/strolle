use crate::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Lights {
    items: [Light; MAX_LIGHTS as _],
    len: PadU32,
}

impl Lights {
    pub fn get(&self, id: usize) -> Light {
        unsafe { *self.items.get_unchecked(id) }
    }

    pub fn len(&self) -> usize {
        self.len.value as _
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Lights {
    pub fn push(&mut self, item: Light) {
        self.items[self.len.value as usize] = item;
        self.len += 1;
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Light {
        &mut self.items[id]
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for Lights {
    fn default() -> Self {
        Self::zeroed()
    }
}
