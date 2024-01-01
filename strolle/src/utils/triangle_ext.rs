use glam::{Vec2, Vec3, Vec4};

use crate::{gpu, BoundingBox};

pub trait TriangleExt {
    fn positions(&self) -> [Vec3; 3];
    fn normals(&self) -> [Vec3; 3];
    fn tangents(&self) -> [Vec4; 3];
    fn uvs(&self) -> [Vec2; 3];

    fn center(&self) -> Vec3 {
        self.positions().iter().copied().sum::<Vec3>() / 3.0
    }

    fn bounds(&self) -> BoundingBox {
        self.positions().iter().copied().collect()
    }
}

impl TriangleExt for gpu::Triangle {
    fn positions(&self) -> [Vec3; 3] {
        [self.position0(), self.position1(), self.position2()]
    }

    fn normals(&self) -> [Vec3; 3] {
        [self.normal0(), self.normal1(), self.normal2()]
    }

    fn tangents(&self) -> [Vec4; 3] {
        // TODO
        [Vec4::ZERO, Vec4::ZERO, Vec4::ZERO]
    }

    fn uvs(&self) -> [Vec2; 3] {
        [self.uv0(), self.uv1(), self.uv2()]
    }
}
