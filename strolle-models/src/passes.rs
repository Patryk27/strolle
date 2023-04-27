use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DenoisingPassParams {
    pub seed: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DrawingPassParams {
    pub viewport_mode: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct RasterPassParams {
    pub material_id: u32,
    pub has_normal_map: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct RayShadingPassParams {
    pub frame: u32,
    pub seed: u32,
    pub needs_direct_lightning: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct VoxelPaintingPassParams {
    pub frame: u32,
    pub seed: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct VoxelShadingPassParams {
    pub frame: u32,
    pub seed: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct VoxelTracingPassParams {
    pub frame: u32,
    pub seed: u32,
}
