use bytemuck::{Pod, Zeroable};
use glam::{vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{Hit, Ray};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
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
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        n0: Vec3,
        n1: Vec3,
        n2: Vec3,
    ) -> Self {
        Self {
            d0: v0.extend(Default::default()),
            d1: v1.extend(Default::default()),
            d2: v2.extend(Default::default()),
            d3: n0.extend(Default::default()),
            d4: n1.extend(Default::default()),
            d5: n2.extend(Default::default()),
        }
    }

    pub fn v0(&self) -> Vec3 {
        self.d0.xyz()
    }

    pub fn v1(&self) -> Vec3 {
        self.d1.xyz()
    }

    pub fn v2(&self) -> Vec3 {
        self.d2.xyz()
    }

    pub fn n0(&self) -> Vec3 {
        self.d3.xyz()
    }

    pub fn n1(&self) -> Vec3 {
        self.d4.xyz()
    }

    pub fn n2(&self) -> Vec3 {
        self.d5.xyz()
    }

    pub fn with_transform(mut self, mat: glam::Mat4) -> Self {
        self.d0 = mat.transform_point3(self.d0.xyz()).extend(self.d0.w);
        self.d1 = mat.transform_point3(self.d1.xyz()).extend(self.d1.w);
        self.d2 = mat.transform_point3(self.d2.xyz()).extend(self.d2.w);
        self.d3 = mat.transform_vector3(self.d3.xyz()).extend(self.d3.w);
        self.d4 = mat.transform_vector3(self.d4.xyz()).extend(self.d4.w);
        self.d5 = mat.transform_vector3(self.d5.xyz()).extend(self.d5.w);
        self
    }

    // Following the Möller-Trumbore algorithm
    pub fn hit(self, ray: Ray) -> Hit {
        let v0v1 = self.v1() - self.v0();
        let v0v2 = self.v2() - self.v0();
        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);
        let inv_det = 1.0 / det;
        let tvec = ray.origin() - self.v0();
        let u = tvec.dot(pvec) * inv_det;
        let qvec = tvec.cross(v0v1);
        let v = ray.direction().dot(qvec) * inv_det;
        let distance = v0v2.dot(qvec) * inv_det;

        if (u < 0.0) | (u > 1.0) | (v < 0.0) | (u + v > 1.0) | (distance < 0.0)
        {
            return Hit::none();
        }

        let point = ray.origin() + ray.direction() * (distance - 0.01);

        let normal =
            (u * self.n1() + v * self.n2() + (1.0 - u - v) * self.n0())
                .normalize();

        Hit {
            distance,
            uv: vec2(u, v),
            point,
            normal,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Triangle {
    pub fn vertices(&self) -> [Vec3; 3] {
        [self.v0(), self.v1(), self.v2()]
    }

    pub fn center(&self) -> Vec3 {
        self.vertices().iter().sum::<Vec3>() / 3.0
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
