use glam::Vec3Swizzles;
use spirv_std::glam::{Mat3, Mat3A, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::gpu;

#[derive(Clone, Debug, Default)]
pub struct Triangle {
    positions: [Vec3; 3],
    normals: [Vec3; 3],
    uvs: [Vec2; 3],
    tangents: [Vec4; 3],
}

impl Triangle {
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

    pub(crate) fn with_transform(&self, mat: Mat4) -> Self {
        let positions =
            self.positions.map(|vertex| mat.transform_point3(vertex));

        let normals = {
            // Transforming normals requires inversing and transposing the
            // matrix in order to get correct results under scaling, see:
            //
            // https://paroj.github.io/gltut/Illumination/Tut09%20Normal%20Transformation.html
            let mat = mat.inverse().transpose();

            self.normals
                .map(|normal| mat.transform_vector3(normal).normalize())
        };

        let tangents = {
            let sign = if Mat3A::from_mat4(mat).determinant().is_sign_positive()
            {
                1.0
            } else {
                -1.0
            };

            self.tangents.map(|tangent| {
                (Mat3::from_mat4(mat) * tangent.xyz())
                    .normalize()
                    .extend(tangent.w * sign)
            })
        };

        Self {
            positions,
            normals,
            uvs: self.uvs,
            tangents,
        }
    }

    pub(crate) fn serialize(&self) -> gpu::Triangle {
        gpu::Triangle {
            // First vertex
            d0: self.positions[0].xyz().extend(self.uvs[0].x),
            d1: self.normals[0].xyz().extend(self.uvs[0].y),
            d2: self.tangents[0],

            // Second vertex
            d3: self.positions[1].xyz().extend(self.uvs[1].x),
            d4: self.normals[1].xyz().extend(self.uvs[1].y),
            d5: self.tangents[1],

            // Third vertex
            d6: self.positions[2].xyz().extend(self.uvs[2].x),
            d7: self.normals[2].xyz().extend(self.uvs[2].y),
            d8: self.tangents[2],
        }
    }
}
