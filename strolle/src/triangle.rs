use glam::{Vec2, Vec3, Vec3Swizzles, Vec4};

use crate::gpu;
use crate::utils::{BoundingBox, ToGpu};

#[derive(Clone, Debug)]
pub struct Triangle {
    pub positions: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub uvs: [Vec2; 3],
    pub tangents: [Vec4; 3],
}

impl Triangle {
    pub fn center(&self) -> Vec3 {
        self.positions.iter().sum::<Vec3>() / 3.0
    }

    pub fn bounds(&self) -> BoundingBox {
        self.positions.iter().copied().collect()
    }

    pub fn serialize(&self) -> gpu::Triangle {
        gpu::Triangle {
            d0: self.positions[0].xyz().extend(self.uvs[0].x).to_gpu(),
            d1: self.normals[0].xyz().extend(self.uvs[0].y).to_gpu(),
            d2: self.tangents[0].to_gpu(),

            d3: self.positions[1].xyz().extend(self.uvs[1].x).to_gpu(),
            d4: self.normals[1].xyz().extend(self.uvs[1].y).to_gpu(),
            d5: self.tangents[1].to_gpu(),

            d6: self.positions[2].xyz().extend(self.uvs[2].x).to_gpu(),
            d7: self.normals[2].xyz().extend(self.uvs[2].y).to_gpu(),
            d8: self.tangents[2].to_gpu(),
        }
    }
}
