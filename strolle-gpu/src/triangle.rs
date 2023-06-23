use bytemuck::{Pod, Zeroable};
use glam::{vec2, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Hit, Ray};

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
pub struct Triangle {
    pub d0: Vec4,
    pub d1: Vec4,
    pub d2: Vec4,
    pub d3: Vec4,
    pub d4: Vec4,
    pub d5: Vec4,
    pub d6: Vec4,
    pub d7: Vec4,
    pub d8: Vec4,
}

impl Triangle {
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

    pub fn positions(&self) -> [Vec3; 3] {
        [self.position0(), self.position1(), self.position2()]
    }

    pub fn center(&self) -> Vec3 {
        self.positions().into_iter().sum::<Vec3>() / 3.0
    }

    pub fn hit(&self, ray: Ray, hit: &mut Hit) -> bool {
        let v0v1 = self.position1() - self.position0();
        let v0v2 = self.position2() - self.position0();

        // ---

        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);

        if det.abs() < f32::EPSILON {
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
