use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
pub struct Info {
    pub world_bvh_ptr: u32,
    pub light_count: u32,
}

impl Info {
    pub fn is_world_empty(&self) -> bool {
        self.world_bvh_ptr == 0
    }
}
