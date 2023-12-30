use glam::{Affine3A, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{gpu, BoundingBox, MeshTriangle};

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

    fn with_xform(&self, xform: Affine3A, xform_inv_trans: Mat4) -> Self;
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

    fn with_xform(&self, xform: Affine3A, xform_inv_trans: Mat4) -> Self {
        let positions = self
            .positions()
            .map(|vertex| xform.transform_point3(vertex));

        let normals = self.normals().map(|normal| {
            xform_inv_trans.transform_vector3(normal).normalize()
        });

        let tangents = {
            let sign = if xform.matrix3.determinant().is_sign_positive() {
                1.0
            } else {
                -1.0
            };

            self.tangents().map(|tangent| {
                (xform.matrix3 * tangent.xyz())
                    .normalize()
                    .extend(tangent.w * sign)
            })
        };

        MeshTriangle {
            positions,
            normals,
            uvs: self.uvs(),
            tangents,
        }
        .serialize()
    }
}
