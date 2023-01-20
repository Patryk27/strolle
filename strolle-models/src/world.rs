use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct World {
    pub triangle_count: u32,
    pub light_count: u32,
}
