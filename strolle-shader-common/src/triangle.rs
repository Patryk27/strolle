use crate::*;

/// # Memory model
///
/// ```
/// v0.x = vertex 0 (x; f32)
/// v0.y = vertex 0 (y; f32)
/// v0.z = vertex 0 (z; f32)
/// v0.w (bits 0..16) = material id (u16)
///
/// v1.x = vertex 1 (x; f32)
/// v1.y = vertex 1 (y; f32)
/// v1.z = vertex 1 (z; f32)
/// v1.w = alpha channel (0..=1.0; f32)
///
/// v2.x = vertex 2 (x; f32)
/// v2.y = vertex 2 (y; f32)
/// v2.z = vertex 3 (z; f32)
/// v2.w (bit 0) = casts shadows (bool)
/// v2.w (bit 1) = uv-transparent (bool)
/// v2.w (bit 2) = two-sided (bool)
/// v2.w (bits 15..32) = uv divisor (u16)
/// ```
#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Triangle {
    v0: Vec4,
    v1: Vec4,
    v2: Vec4,
}

impl Triangle {
    const CASTS_SHADOWS_MASK: u32 = 1 << 0;
    const UV_TRANSPARENCY_MASK: u32 = 1 << 1;
    const DOUBLE_SIDED_MASK: u32 = 1 << 2;

    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, mat_id: MaterialId) -> Self {
        Self {
            v0: v0.extend(f32::from_bits(mat_id.get() as _)),
            v1: v1.extend(1.0),
            v2: v2.extend(0.0),
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

    pub fn alpha(&self) -> f32 {
        self.v1.w
    }

    pub fn material_id(&self) -> MaterialId {
        MaterialId::new(self.v0.w.to_bits() as _)
    }

    pub fn casts_shadows(&self) -> bool {
        self.v2.w.to_bits() & Self::CASTS_SHADOWS_MASK > 0
    }

    pub fn has_uv_transparency(&self) -> bool {
        self.v2.w.to_bits() & Self::UV_TRANSPARENCY_MASK > 0
    }

    pub fn double_sided(&self) -> bool {
        self.v2.w.to_bits() & Self::DOUBLE_SIDED_MASK > 0
    }

    pub fn uv_divisor(&self) -> Vec2 {
        let w = self.v2.w.to_bits() >> 16;
        let u = w >> 8;
        let v = w & ((1 << 8) - 1);

        vec2(1.0 / (u as f32), 1.0 / (v as f32))
    }

    pub fn hit(self, ray: Ray, culling: bool) -> Hit {
        // Following the Möller-Trumbore algorithm

        let v0v1 = (self.v1 - self.v0).truncate();
        let v0v2 = (self.v2 - self.v0).truncate();
        let pvec = ray.direction().cross(v0v2);
        let det = v0v1.dot(pvec);

        if culling && !self.double_sided() {
            if det < f32::EPSILON {
                return Hit::none();
            }
        } else {
            if det.abs() < f32::EPSILON {
                return Hit::none();
            }
        }

        let inv_det = 1.0 / det;
        let tvec = ray.origin() - self.v0.truncate();
        let u = tvec.dot(pvec) * inv_det;
        let qvec = tvec.cross(v0v1);
        let v = ray.direction().dot(qvec) * inv_det;
        let t = v0v2.dot(qvec) * inv_det;

        if (u < 0.0) | (u > 1.0) | (v < 0.0) | (u + v > 1.0) | (t < 0.0) {
            return Hit::none();
        }

        let uv_divisor = self.uv_divisor();

        let normal = {
            let n = v0v1.cross(v0v2).normalize();

            if det < 0.0 {
                -n
            } else {
                n
            }
        };

        Hit {
            t,
            uv: vec2(u, v).extend(uv_divisor.x).extend(uv_divisor.y),
            ray,
            point: ray.origin() + ray.direction() * (t - 0.01),
            normal,
            tri_id: TriangleId::new_static(0).into_any(),
            mat_id: self.material_id(),
            alpha: self.alpha(),
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Triangle {
    pub fn with_alpha(mut self, val: f32) -> Self {
        self.v1.w = val;
        self
    }

    pub fn with_transform(mut self, val: Mat4) -> Self {
        self.v0 = math::transform(self.v0.xyz(), val).extend(self.v0.w);
        self.v1 = math::transform(self.v1.xyz(), val).extend(self.v1.w);
        self.v2 = math::transform(self.v2.xyz(), val).extend(self.v2.w);
        self
    }

    pub fn with_casts_shadows(mut self, val: bool) -> Self {
        let mut w = self.v2.w.to_bits();

        if val {
            w |= Self::CASTS_SHADOWS_MASK;
        } else {
            w &= !Self::CASTS_SHADOWS_MASK;
        }

        self.v2.w = f32::from_bits(w);
        self
    }

    pub fn with_uv_transparency(mut self, val: bool) -> Self {
        let mut w = self.v2.w.to_bits();

        if val {
            w |= Self::UV_TRANSPARENCY_MASK;
        } else {
            w &= !Self::UV_TRANSPARENCY_MASK;
        }

        self.v2.w = f32::from_bits(w);
        self
    }

    pub fn with_double_sided(mut self, val: bool) -> Self {
        let mut w = self.v2.w.to_bits();

        if val {
            w |= Self::DOUBLE_SIDED_MASK;
        } else {
            w &= !Self::DOUBLE_SIDED_MASK;
        }

        self.v2.w = f32::from_bits(w);
        self
    }

    pub fn with_uv_divisor(mut self, u: u8, v: u8) -> Self {
        let mut w = self.v2.w.to_bits();

        let u = u as u32;
        let v = v as u32;

        w |= (u << (3 * 8)) | (v << (2 * 8));

        self.v2.w = f32::from_bits(w);
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
pub struct TriangleId<T>(T, usize);

impl<T> TriangleId<T> {
    pub(crate) fn new(provenance: T, id: usize) -> Self {
        Self(provenance, id)
    }

    pub(crate) fn unpack(self) -> (T, usize) {
        (self.0, self.1)
    }
}

impl TriangleId<StaticTriangle> {
    pub fn new_static(id: usize) -> Self {
        Self(StaticTriangle, id)
    }

    pub fn get(self) -> usize {
        self.1
    }

    pub fn into_any(self) -> TriangleId<AnyTriangle> {
        TriangleId::new(AnyTriangle, self.1)
    }
}

impl TriangleId<DynamicTriangle> {
    pub fn new_dynamic(id: usize) -> Self {
        Self(DynamicTriangle, id)
    }

    pub fn get(self) -> usize {
        self.1
    }

    pub fn into_any(self) -> TriangleId<AnyTriangle> {
        TriangleId::new(AnyTriangle, MAX_STATIC_TRIANGLES + self.1)
    }
}

#[cfg(not(target_arch = "spirv"))]
impl fmt::Display for TriangleId<StaticTriangle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StaticTriangle;

#[derive(Copy, Clone, Debug)]
pub struct DynamicTriangle;

#[derive(Copy, Clone, Debug)]
pub struct AnyTriangle;
