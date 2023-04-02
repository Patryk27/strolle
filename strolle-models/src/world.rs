use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use crate::{VoxelId, VOXELS_MAP_LENGTH, VOXEL_SIZE};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct World {
    pub light_count: u32,
    pub pad1: u32,
    pub pad2: u32,
    pub pad3: u32,
    pub min_aabb: Vec3,
}

impl World {
    // TODO use more bits for x & z
    pub fn voxelize(&self, position: Vec3, normal: Vec3) -> VoxelId {
        fn expand_bits(mut x: u32) -> u32 {
            x &= 0x3ff;
            x = (x | x << 16) & 0x30000ff;
            x = (x | x << 8) & 0x300f00f;
            x = (x | x << 4) & 0x30c30c3;
            x = (x | x << 2) & 0x9249249;
            x
        }

        let position = ((position - self.min_aabb) / VOXEL_SIZE).as_uvec3();
        let px = expand_bits(position.x);
        let pz = expand_bits(position.z) << 1;
        let py = expand_bits(position.y) << 2;

        let normal = ((normal + Vec3::ONE) * 50.0).as_uvec3();
        let nx = normal.x * 22741;
        let ny = normal.y * 82421;
        let nz = normal.z * 11243;

        let id = ((px | py | pz) ^ (nx ^ ny ^ nz)) % (VOXELS_MAP_LENGTH as u32);

        VoxelId::new(id as u32)
    }
}
