use glam::Vec3;

use crate::{Hit, MaterialId};

#[derive(Clone, Copy)]
pub struct PendingVoxelHit {
    pub point: Vec3,
    pub material_id: MaterialId,
    pub normal: Vec3,
}

impl PendingVoxelHit {
    pub fn from_hit(hit: Hit) -> Self {
        if hit.is_none() {
            Self {
                point: Default::default(),
                material_id: MaterialId::new(0),
                normal: Default::default(),
            }
        } else {
            Self {
                point: hit.point,
                material_id: MaterialId::new(hit.material_id),
                normal: hit.normal, // TODO why not flat_normal?
            }
        }
    }

    pub fn as_hit(&self) -> Hit {
        if self.normal == Default::default() {
            Hit::none()
        } else {
            Hit {
                distance: 0.0, // TODO
                point: self.point,
                normal: self.normal,
                flat_normal: self.normal,
                uv: Default::default(), // TODO (?)
                material_id: self.material_id.get(),
            }
        }
    }
}
