use glam::{Vec3, Vec4, Vec4Swizzles};

use crate::{Voxel, VoxelId};

#[derive(Copy, Clone)]
pub struct VoxelsView<'a> {
    buffer: &'a [Vec4],
}

impl<'a> VoxelsView<'a> {
    pub fn new(buffer: &'a [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, id: VoxelId) -> Voxel {
        let d0 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize()) };
        let d1 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize() + 1) };

        Voxel {
            accum_color: d0.xyz(),
            samples: d0.w,
            point: d1.xyz(),
            frame: d1.w.to_bits(),
        }
    }
}

pub struct VoxelsViewMut<'a> {
    buffer: &'a mut [Vec4],
}

impl<'a> VoxelsViewMut<'a> {
    pub fn new(buffer: &'a mut [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn get(&self, id: VoxelId) -> Voxel {
        VoxelsView::new(self.buffer).get(id)
    }

    pub fn set(&mut self, id: VoxelId, voxel: Voxel) {
        let d0 = voxel.accum_color.extend(voxel.samples);
        let d1 = voxel.point.extend(f32::from_bits(voxel.frame));

        unsafe {
            *self.buffer.get_unchecked_mut(2 * id.get_usize()) = d0;
            *self.buffer.get_unchecked_mut(2 * id.get_usize() + 1) = d1;
        }
    }

    pub fn add_sample(&mut self, id: VoxelId, color: Vec3, weight: f32) {
        unsafe {
            *self.buffer.get_unchecked_mut(2 * id.get_usize()) +=
                (weight * color).extend(weight);
        }
    }
}
