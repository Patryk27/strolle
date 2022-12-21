use crate::*;

/// # Memory model
///
/// ```ignore
/// v0.x = vertex 0 (x; f32)
/// v0.y = vertex 0 (y; f32)
/// v0.z = vertex 0 (z; f32)
/// v0.w (bits 0..16) = material id (u16)
///
/// v1.x = vertex 1 (x; f32)
/// v1.y = vertex 1 (y; f32)
/// v1.z = vertex 1 (z; f32)
///
/// v2.x = vertex 2 (x; f32)
/// v2.y = vertex 2 (y; f32)
/// v2.z = vertex 3 (z; f32)
/// ```
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Triangle {
    // TODO make them private
    pub v0: Vec4,
    pub v1: Vec4,
    pub v2: Vec4,
    pub n0: Vec4,
    pub n1: Vec4,
    pub n2: Vec4,
}

impl Triangle {
    pub fn new(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        n0: Vec3,
        n1: Vec3,
        n2: Vec3,
        mat_id: MaterialId,
    ) -> Self {
        Self {
            v0: v0.extend(f32::from_bits(mat_id.get() as _)),
            v1: v1.extend(1.0),
            v2: v2.extend(0.0),
            n0: n0.extend(0.0),
            n1: n1.extend(0.0),
            n2: n2.extend(0.0),
        }
    }

    #[cfg(not(target_arch = "spirv"))]
    pub fn is_none(self) -> bool {
        self == Self::default()
    }

    #[cfg(not(target_arch = "spirv"))]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }

    pub fn v0(&self) -> Vec3 {
        self.v0.xyz()
    }
    pub fn v1(&self) -> Vec3 {
        self.v1.xyz()
    }

    pub fn v2(&self) -> Vec3 {
        self.v2.xyz()
    }

    pub fn material_id(&self) -> MaterialId {
        MaterialId::new(self.v0.w.to_bits() as _)
    }

    pub fn hit(self, ray: Ray, culling: Culling) -> Hit {
        // Following the Möller-Trumbore algorithm

        let v0v1 = (self.v1 - self.v0).truncate();
        let v0v2 = (self.v2 - self.v0).truncate();
        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);

        if culling.enabled() {
            if det < f32::EPSILON {
                return Hit::none();
            }
        } else if det.abs() < f32::EPSILON {
            return Hit::none();
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin() - self.v0.truncate();
        let u = tvec.dot(pvec) * inv_det;
        let qvec = tvec.cross(v0v1);
        let v = ray.direction().dot(qvec) * inv_det;
        let distance = v0v2.dot(qvec) * inv_det;

        if (u < 0.0) | (u > 1.0) | (v < 0.0) | (u + v > 1.0) | (distance < 0.0)
        {
            return Hit::none();
        }

        let normal = {
            // TODO
            //
            // let n = u * self.n1.xyz()
            //     + v * self.n2.xyz()
            //     + (1.0 - u - v) * self.n0.xyz();

            v0v1.cross(v0v2).normalize()
        };

        Hit {
            dist: distance,
            uv: vec2(u, v),
            ray,
            point: ray.origin() + ray.direction() * (distance - 0.01),
            normal,
            mat_id: self.material_id(),
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Triangle {
    pub fn with_transform(mut self, val: Mat4) -> Self {
        fn transform(v: Vec3, xform: Mat4) -> Vec3 {
            let v = xform * v.extend(1.0);
            Vec3::new(v.x, v.y, v.z)
        }

        self.v0 = transform(self.v0.xyz(), val).extend(self.v0.w);
        self.v1 = transform(self.v1.xyz(), val).extend(self.v1.w);
        self.v2 = transform(self.v2.xyz(), val).extend(self.v2.w);
        self
    }

    pub fn vertices(&self) -> [Vec3; 3] {
        [self.v0(), self.v1(), self.v2()]
    }

    pub fn center(&self) -> Vec3 {
        self.vertices().iter().sum::<Vec3>() / 3.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TriangleId(usize);

impl TriangleId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[cfg(not(target_arch = "spirv"))]
impl fmt::Display for TriangleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
