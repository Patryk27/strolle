use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Pod, Zeroable,
)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Frame(u32);

impl Frame {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn is_gi_tracing(self) -> bool {
        self.0 % 6 < 4
    }

    pub fn is_gi_validation(self) -> bool {
        !self.is_gi_tracing()
    }
}
