use glam::{UVec2, Vec3};

use crate::VoxelId;

#[derive(Copy, Clone)]
pub struct PendingVoxel {
    pub color: Vec3,
    pub frame: u32,
    pub point: Vec3,
    pub voxel_id: VoxelId,
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct PendingVoxelId(u32);

impl PendingVoxelId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn from_xy(width: u32, position: UVec2) -> Self {
        Self::new(position.x + position.y * width)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn get_usize(self) -> usize {
        self.get() as usize
    }
}
