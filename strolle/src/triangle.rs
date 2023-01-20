use spirv_std::glam::{Mat4, Vec2, Vec3};
use strolle_models as gpu;

#[derive(Clone, Debug, Default)]
pub struct Triangle {
    vertices: [Vec3; 3],
    normals: [Vec3; 3],
    uvs: [Vec2; 3],
}

impl Triangle {
    pub fn new(
        vertices: [Vec3; 3],
        normals: [Vec3; 3],
        uvs: [Vec2; 3],
    ) -> Self {
        Self {
            vertices,
            normals,
            uvs,
        }
    }

    pub fn with_vertices(mut self, vertices: [impl Into<Vec3>; 3]) -> Self {
        self.vertices = vertices.map(Into::into);
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

    pub fn vertices(&self) -> [Vec3; 3] {
        self.vertices
    }

    pub fn normals(&self) -> [Vec3; 3] {
        self.normals
    }

    pub fn uvs(&self) -> [Vec2; 3] {
        self.uvs
    }

    pub(crate) fn transformed(&self, mat: Mat4) -> Self {
        let vertices = self.vertices.map(|vertex| mat.transform_point3(vertex));
        let normals = self.normals.map(|normal| mat.transform_vector3(normal));

        Self {
            vertices,
            normals,
            uvs: self.uvs,
        }
    }

    pub(crate) fn serialize(&self) -> gpu::Triangle {
        gpu::Triangle::new(self.vertices, self.normals, self.uvs)
    }
}
