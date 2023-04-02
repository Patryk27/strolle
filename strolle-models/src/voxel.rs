use glam::{Vec3, Vec3Swizzles, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::float::Float;

use crate::VOXEL_SIZE;

#[derive(Copy, Clone)]
pub struct Voxel {
    pub accum_color: Vec3,
    pub samples: f32,
    pub point: Vec3,
    pub frame: u32,
}

impl Voxel {
    pub fn is_fresh(&self, frame: u32) -> bool {
        frame - self.frame < 10
    }

    pub fn is_nearby(&self, point: Vec3) -> bool {
        self.point.distance_squared(point).abs() < 10.0 * VOXEL_SIZE
    }

    pub fn color(&self) -> Vec3 {
        self.accum_color.xyz() / self.samples
    }

    pub fn scolor(&self) -> Vec4 {
        self.accum_color.extend(self.samples)
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct VoxelId(u32);

impl VoxelId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn get_usize(self) -> usize {
        self.get() as usize
    }
}
