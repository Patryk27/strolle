use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{GBufferEntry, MaterialId, Normal, Ray};

#[derive(Clone, Copy)]
pub struct Hit {
    pub origin: Vec3,
    pub direction: Vec3,
    pub point: Vec3,
    pub gbuffer: GBufferEntry,
}

impl Hit {
    pub fn from_direct(ray: Ray, point: Vec3, gbuffer: GBufferEntry) -> Self {
        Self {
            origin: ray.origin(),
            direction: ray.direction(),
            point,
            gbuffer,
        }
    }

    pub fn from_indirect(ray: Ray, gbuffer: GBufferEntry) -> Self {
        Self {
            origin: ray.origin(),
            direction: ray.direction(),
            point: ray.origin() + ray.direction() * gbuffer.depth,
            gbuffer,
        }
    }

    pub fn is_some(&self) -> bool {
        self.gbuffer.is_some()
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
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
    /// How far to move a hit point away from its surface to avoid
    /// self-intersection when casting shadow rays.
    ///
    /// This constant cannot be zero (because then every object would cast
    /// shadows onto itself), but it cannot be too high either (because then
    /// shadows would feel off, flying on surfaces instead of being attached to
    /// them).
    pub const NUDGE_OFFSET: f32 = 0.001;

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

    pub fn pack(&self) -> [Vec4; 2] {
        let d0 = self.point.extend(f32::from_bits(self.material_id.get()));

        let d1 = Normal::encode(self.normal)
            .extend(self.uv.x)
            .extend(self.uv.y);

        [d0, d1]
    }

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }
}
