use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{MaterialId, Normal};

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: MaterialId,
}

impl Hit {
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

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn serialize(&self) -> [Vec4; 2] {
        let d0 = self.point.extend(f32::from_bits(self.material_id.get()));

        let d1 = Normal::encode(self.normal)
            .extend(self.uv.x)
            .extend(self.uv.y);

        [d0, d1]
    }

    pub fn deserialize(d0: Vec4, d1: Vec4) -> Self {
        if d0.xyz() == Default::default() {
            Self::none()
        } else {
            let normal = Normal::decode(d1.xy());
            let point = d0.xyz() + normal * Self::NUDGE_OFFSET;

            Self {
                distance: 0.0,
                point,
                normal,
                uv: d1.zw(),
                material_id: MaterialId::new(d0.w.to_bits()),
            }
        }
    }

    pub fn deserialize_point(d0: Vec4) -> Vec3 {
        d0.xyz()
    }
}
