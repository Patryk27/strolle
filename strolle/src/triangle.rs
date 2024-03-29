use glam::Vec3Swizzles;
use spirv_std::glam::{Vec2, Vec3, Vec4};

use crate::gpu;
use crate::utils::BoundingBox;

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
            d0: self.positions[0].xyz().extend(self.uvs[0].x),
            d1: self.normals[0].xyz().extend(self.uvs[0].y),
            d2: self.tangents[0],

            d3: self.positions[1].xyz().extend(self.uvs[1].x),
            d4: self.normals[1].xyz().extend(self.uvs[1].y),
            d5: self.tangents[1],

            d6: self.positions[2].xyz().extend(self.uvs[2].x),
            d7: self.normals[2].xyz().extend(self.uvs[2].y),
            d8: self.tangents[2],
        }
    }
}
