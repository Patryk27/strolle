use glam::{Vec4, Vec4Swizzles};

use crate::{
    MaterialId, PendingVoxelHit, PendingVoxelId, PendingVoxelsViewMut,
};

pub struct PendingVoxelHitsViewMut<'a> {
    buffer: &'a mut [Vec4],
}

impl<'a> PendingVoxelHitsViewMut<'a> {
    pub fn new(buffer: &'a mut [Vec4]) -> Self {
        Self { buffer }
    }

    pub fn set(&mut self, id: PendingVoxelId, voxel: PendingVoxelHit) {
        let d0 = voxel.point.extend(f32::from_bits(voxel.material_id.get()));
        let d1 = voxel.normal.extend(Default::default());

        unsafe {
            *self.buffer.get_unchecked_mut(2 * id.get_usize()) = d0;
            *self.buffer.get_unchecked_mut(2 * id.get_usize() + 1) = d1;
        }
    }

    pub fn get(&self, id: PendingVoxelId) -> PendingVoxelHit {
        let d0 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize()) };
        let d1 = unsafe { *self.buffer.get_unchecked(2 * id.get_usize() + 1) };

        PendingVoxelHit {
            point: d0.xyz(),
            material_id: MaterialId::new(d0.w.to_bits()),
            normal: d1.xyz(),
        }
    }

    pub fn as_pending_voxels_view_mut(self) -> PendingVoxelsViewMut<'a> {
        PendingVoxelsViewMut::new(self.buffer)
    }
}
