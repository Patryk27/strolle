use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{GBufferEntry, MaterialId, Normal, Ray, Surface, Vec3Ext};

#[derive(Clone, Copy, Default)]
pub struct Hit {
    pub origin: Vec3,
    pub dir: Vec3,
    pub point: Vec3,
    pub gbuffer: GBufferEntry,
}

impl Hit {
    /// How far to move a hit point away from its surface to avoid
    /// self-intersection when casting shadow rays
    pub const NUDGE_OFFSET: f32 = 0.01;

    pub fn new(ray: Ray, gbuffer: GBufferEntry) -> Self {
        Self {
            origin: ray.origin(),
            dir: ray.dir(),
            point: ray.at(gbuffer.depth - Self::NUDGE_OFFSET),
            gbuffer,
        }
    }

    pub fn is_some(self) -> bool {
        self.gbuffer.is_some()
    }

    pub fn is_none(self) -> bool {
        !self.is_some()
    }

    pub fn as_surface(self) -> Surface {
        Surface {
            normal: self.gbuffer.normal,
            depth: self.gbuffer.depth,
            roughness: self.gbuffer.roughness,
        }
    }

    pub fn kernel_basis(
        normal: Vec3,
        direction: Vec3,
        roughness: f32,
        size: f32,
    ) -> (Vec3, Vec3) {
        fn dominant_direction(n: Vec3, v: Vec3, roughness: f32) -> Vec3 {
            let f = (1.0 - roughness) * ((1.0 - roughness).sqrt() + roughness);
            let r = (-v).reflect(n);

            n.lerp(r, f).normalize()
        }

        let t;
        let b;

        if roughness == 1.0 {
            (t, b) = normal.any_orthonormal_pair();
        } else {
            let d = dominant_direction(normal, -direction, roughness);
            let r = (-d).reflect(normal);

            t = normal.cross(r).normalize();
            b = r.cross(t);
        }

        (t * size, b * size)
    }
}

#[derive(Clone, Copy)]
pub struct TriangleHit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: MaterialId,
}

impl TriangleHit {
    pub fn none() -> Self {
        Self {
            distance: f32::MAX,
            point: Default::default(),
            normal: Default::default(),
            uv: Default::default(),
            material_id: MaterialId::new(0),
        }
    }

    pub fn unpack([d0, d1]: [Vec4; 2]) -> Self {
        if d0.xyz() == Default::default() {
            Self::none()
        } else {
            let normal = Normal::decode(d1.xy());
            let point = d0.xyz();

            Self {
                distance: 0.0,
                point,
                normal,
                uv: d1.zw(),
                material_id: MaterialId::new(d0.w.to_bits()),
            }
        }
    }

    pub fn pack(self) -> [Vec4; 2] {
        let d0 = self.point.extend(f32::from_bits(self.material_id.get()));

        let d1 = Normal::encode(self.normal)
            .extend(self.uv.x)
            .extend(self.uv.y);

        [d0, d1]
    }

    pub fn is_some(self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(self) -> bool {
        !self.is_some()
    }
}
