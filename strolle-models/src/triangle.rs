use bytemuck::{Pod, Zeroable};
use glam::{vec2, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

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
    d6: Vec4,
    d7: Vec4,
    d8: Vec4,
}

impl Triangle {
    pub fn new(
        positions: [Vec3; 3],
        normals: [Vec3; 3],
        uvs: [Vec2; 3],
        tangents: [Vec4; 3],
    ) -> Self {
        Self {
            // First vertex
            d0: positions[0].xyz().extend(uvs[0].x),
            d1: normals[0].xyz().extend(uvs[0].y),
            d2: tangents[0],

            // Second vertex
            d3: positions[1].xyz().extend(uvs[1].x),
            d4: normals[1].xyz().extend(uvs[1].y),
            d5: tangents[1],

            // Third vertex
            d6: positions[2].xyz().extend(uvs[2].x),
            d7: normals[2].xyz().extend(uvs[2].y),
            d8: tangents[2],
        }
    }

    pub fn position0(&self) -> Vec3 {
        self.d0.xyz()
    }

    pub fn normal0(&self) -> Vec3 {
        self.d1.xyz()
    }

    pub fn uv0(&self) -> Vec2 {
        vec2(self.d0.w, self.d1.w)
    }

    pub fn position1(&self) -> Vec3 {
        self.d3.xyz()
    }

    pub fn normal1(&self) -> Vec3 {
        self.d4.xyz()
    }

    pub fn uv1(&self) -> Vec2 {
        vec2(self.d3.w, self.d4.w)
    }

    pub fn position2(&self) -> Vec3 {
        self.d6.xyz()
    }

    pub fn normal2(&self) -> Vec3 {
        self.d7.xyz()
    }

    pub fn uv2(&self) -> Vec2 {
        vec2(self.d6.w, self.d7.w)
    }

    pub fn vertices(&self) -> [Vec3; 3] {
        [self.position0(), self.position1(), self.position2()]
    }

    pub fn center(&self) -> Vec3 {
        self.vertices().into_iter().sum::<Vec3>() / 3.0
    }

    pub fn hit(&self, ray: Ray, hit: &mut Hit) -> bool {
        let v0v1 = self.position1() - self.position0();
        let v0v2 = self.position2() - self.position0();

        // ---

        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);

        if det < f32::EPSILON {
            return false;
        }

        // ---

        let inv_det = 1.0 / det;
        let tvec = ray.origin() - self.position0();
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
