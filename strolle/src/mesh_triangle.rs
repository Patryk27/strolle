use glam::{Affine3A, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Triangle;

#[derive(Clone, Debug, Default)]
pub struct MeshTriangle {
    positions: [Vec3; 3],
    normals: [Vec3; 3],
    uvs: [Vec2; 3],
    tangents: [Vec4; 3],
}

impl MeshTriangle {
    pub fn with_positions(mut self, positions: [impl Into<Vec3>; 3]) -> Self {
        self.positions = positions.map(Into::into);
        self
    }

    pub fn with_normals(mut self, normals: [Vec3; 3]) -> Self {
        self.normals = normals;
        self
    }

    pub fn with_uvs(mut self, uvs: [Vec2; 3]) -> Self {
        self.uvs = uvs;
        self
    }

    pub fn with_tangents(mut self, tangents: [Vec4; 3]) -> Self {
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

    pub(crate) fn build(
        &self,
        xform: Affine3A,
        xform_inv: Affine3A,
    ) -> Triangle {
        let positions: [Vec3; 3] = self
            .positions
            .map(|vertex: Vec3| -> Vec3 { xform.transform_point3(vertex) });

        let normals = {
            // Transforming normals requires inversing and transposing the
            // matrix in order to get correct results under scaling, see:
            //
            // https://paroj.github.io/gltut/Illumination/Tut09%20Normal%20Transformation.html
            let mat = Mat4::from(xform_inv).transpose();

            self.normals
                .map(|normal| mat.transform_vector3(normal).normalize())
        };

        let tangents = {
            let sign = if xform.matrix3.determinant().is_sign_positive() {
                1.0
            } else {
                -1.0
            };

            self.tangents.map(|tangent| {
                (xform.matrix3 * tangent.xyz())
                    .normalize()
                    .extend(tangent.w * sign)
            })
        };

        let uvs = self.uvs.map(|uv| uv);

        Triangle {
            positions,
            normals,
            uvs,
            tangents,
        }
    }
}
