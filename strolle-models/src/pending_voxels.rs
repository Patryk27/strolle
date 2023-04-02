use glam::{Vec4, Vec4Swizzles};

use crate::{PendingVoxel, PendingVoxelId, VoxelId};

#[derive(Copy, Clone)]
pub struct PendingVoxelsView<'a> {
    buffer: &'a [Vec4],
}

impl<'a> PendingVoxelsView<'a> {
    pub fn new(buffer: &'a [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, id: PendingVoxelId) -> PendingVoxel {
        let d0 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize()) };
        let d1 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize() + 1) };

        PendingVoxel {
            color: d0.xyz(),
            frame: d0.w.to_bits(),
            point: d1.xyz(),
            voxel_id: VoxelId::new(d1.w.to_bits()),
        }
    }
}

pub struct PendingVoxelsViewMut<'a> {
    buffer: &'a mut [Vec4],
}

impl<'a> PendingVoxelsViewMut<'a> {
    pub fn new(buffer: &'a mut [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn set(&mut self, id: PendingVoxelId, voxel: PendingVoxel) {
        let d0 = voxel.color.extend(f32::from_bits(voxel.frame));
        let d1 = voxel.point.extend(f32::from_bits(voxel.voxel_id.get()));

        unsafe {
            *self.buffer.get_unchecked_mut(2 * id.get_usize()) = d0;
            *self.buffer.get_unchecked_mut(2 * id.get_usize() + 1) = d1;
        }
    }
}
