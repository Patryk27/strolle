use glam::{vec4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::Ray;

#[derive(Copy, Clone)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: u32,
    pub traversed_nodes: u32,
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
            traversed_nodes: Default::default(),
        }
    }

    pub fn is_some(&self) -> bool {
        self.distance < f32::MAX
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn serialize(self) -> [Vec4; 2] {
        let d0 = vec4(
            self.normal.x,
            self.normal.y,
            self.normal.z,
            f32::from_bits(self.material_id),
        );

        let d1 = vec4(
            self.uv.x,
            self.uv.y,
            self.distance,
            f32::from_bits(self.traversed_nodes),
        );

        [d0, d1]
    }

    pub fn deserialize([d0, d1]: [Vec4; 2], ray: Ray) -> Self {
        let normal = d0.xyz();
        let material_id = d0.w.to_bits();
        let uv = d1.xy();
        let distance = d1.z;
        let traversed_nodes = d1.w.to_bits();

        let point =
            ray.origin() + ray.direction() * (distance - Self::DISTANCE_OFFSET);

        Self {
            distance,
            point,
            normal,
            uv,
            material_id,
            traversed_nodes,
        }
    }
}
