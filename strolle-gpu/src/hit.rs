use glam::{Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Normal;

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: u32,
}

impl Hit {
    pub const DISTANCE_OFFSET: f32 = 0.01;

    pub fn none() -> Self {
        Self {
            distance: f32::MAX,
            point: Default::default(),
            normal: Default::default(),
            uv: Default::default(),
            material_id: Default::default(),
        }
    }

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn serialize(&self) -> [Vec4; 2] {
        let d0 = self.point.extend(f32::from_bits(self.material_id));

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

            // We're moving the hit-point a few units away from the surface to
            // prevent self-intersecting when checking for shadows later
            let point = d0.xyz() + normal * 0.001;

            Self {
                distance: 0.0,
                point,
                normal,
                uv: d1.zw(),
                material_id: d0.w.to_bits(),
            }
        }
    }

    pub fn deserialize_point(d0: Vec4) -> Vec3 {
        d0.xyz()
    }
}