use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{Hit, Ray};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct Triangle {
    d0: Vec4,
    d1: Vec4,
    d2: Vec4,
    d3: Vec4,
    d4: Vec4,
    d5: Vec4,
}

impl Triangle {
    pub fn new(
        vertices: [Vec3; 3],
        normals: [Vec3; 3],
        uvs: [Vec2; 3],
    ) -> Self {
        Self {
            d0: vec4(vertices[0].x, vertices[0].y, vertices[0].z, normals[0].x),
            d1: vec4(vertices[1].x, vertices[1].y, vertices[1].z, normals[0].y),
            d2: vec4(vertices[2].x, vertices[2].y, vertices[2].z, normals[0].z),
            d3: vec4(normals[1].x, normals[1].y, normals[1].z, uvs[0].x),
            d4: vec4(normals[2].x, normals[2].y, normals[2].z, uvs[0].y),
            d5: vec4(uvs[1].x, uvs[1].y, uvs[2].x, uvs[2].y),
        }
    }

    pub fn vertex0(&self) -> Vec3 {
        self.d0.xyz()
    }

    pub fn vertex1(&self) -> Vec3 {
        self.d1.xyz()
    }

    pub fn vertex2(&self) -> Vec3 {
        self.d2.xyz()
    }

    pub fn normal0(&self) -> Vec3 {
        vec3(self.d0.w, self.d1.w, self.d2.w)
    }

    pub fn normal1(&self) -> Vec3 {
        self.d3.xyz()
    }

    pub fn normal2(&self) -> Vec3 {
        self.d4.xyz()
    }

    pub fn uv0(&self) -> Vec2 {
        vec2(self.d3.w, self.d4.w)
    }

    pub fn uv1(&self) -> Vec2 {
        self.d5.xy()
    }

    pub fn uv2(&self) -> Vec2 {
        self.d5.zw()
    }

    pub fn vertices(&self) -> [Vec3; 3] {
        [self.vertex0(), self.vertex1(), self.vertex2()]
    }

    pub fn center(&self) -> Vec3 {
        self.vertices().into_iter().sum::<Vec3>() / 3.0
    }

    pub fn hit(&self, ray: Ray, hit: &mut Hit) -> bool {
        let v0v1 = self.vertex1() - self.vertex0();
        let v0v2 = self.vertex2() - self.vertex0();

        // ---

        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);

        if det < f32::EPSILON {
            return false;
        }

        // ---

        let inv_det = 1.0 / det;
        let tvec = ray.origin() - self.vertex0();
        let u = tvec.dot(pvec) * inv_det;
        let qvec = tvec.cross(v0v1);
        let v = ray.direction().dot(qvec) * inv_det;
        let distance = v0v2.dot(qvec) * inv_det;

        if (u < 0.0)
            | (u > 1.0)
            | (v < 0.0)
            | (u + v > 1.0)
            | (distance <= 0.0)
            | (distance >= hit.distance)
        {
            return false;
        }

        let point =
            ray.origin() + ray.direction() * (distance - Hit::DISTANCE_OFFSET);

        let normal = u * self.normal1()
            + v * self.normal2()
            + (1.0 - u - v) * self.normal0();

        let normal = normal.normalize();

        let uv = self.uv0()
            + (self.uv1() - self.uv0()) * u
            + (self.uv2() - self.uv0()) * v;

        hit.distance = distance;
        hit.point = point;
        hit.normal = normal;
        hit.uv = uv;

        true
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct TriangleId(u32);

impl TriangleId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn get_mut(&mut self) -> &mut u32 {
        &mut self.0
    }
}
