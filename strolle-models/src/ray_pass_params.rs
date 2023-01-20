use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct RayPassParams {
    pub bounce: u32,
    pub tick: u32,
    pub seed: u32,
    pub apply_denoising: u32,
}

impl RayPassParams {
    pub fn is_casting_primary_rays(&self) -> bool {
        self.bounce == 0
    }

    pub fn apply_denoising(&self) -> bool {
        self.apply_denoising == 1
    }
}
