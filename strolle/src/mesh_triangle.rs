use glam::Vec3Swizzles;
use spirv_std::glam::{Vec2, Vec3, Vec4};

use crate::{gpu, BoundingBox};

#[derive(Clone, Debug, Default)]
pub struct MeshTriangle {
    pub(crate) positions: [Vec3; 3],
    pub(crate) normals: [Vec3; 3],
    pub(crate) uvs: [Vec2; 3],
    pub(crate) tangents: [Vec4; 3],
}

impl MeshTriangle {
    pub fn with_positions(mut self, positions: [impl Into<Vec3>; 3]) -> Self {
        self.positions = positions.map(Into::into);
        self
    }

    pub fn with_normals(mut self, normals: [impl Into<Vec3>; 3]) -> Self {
        self.normals = normals.map(Into::into);
        self
    }

    pub fn with_uvs(mut self, uvs: [impl Into<Vec2>; 3]) -> Self {
        self.uvs = uvs.map(Into::into);
        self
    }

    pub fn with_tangents(mut self, tangents: [impl Into<Vec4>; 3]) -> Self {
        self.tangents = tangents.map(Into::into);
        self
    }

    pub fn positions(&self) -> [Vec3; 3] {
        self.positions
    }

    pub fn normals(&self) -> [Vec3; 3] {
        self.normals
    }

    pub fn uvs(&self) -> [Vec2; 3] {
        self.uvs
    }

    pub(crate) fn center(&self) -> Vec3 {
        self.positions.iter().sum::<Vec3>() / 3.0
    }

    pub(crate) fn bounds(&self) -> BoundingBox {
        self.positions.iter().copied().collect()
    }

    pub(crate) fn serialize(&self) -> gpu::Triangle {
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
