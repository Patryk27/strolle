use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, Vec4Swizzles};

use crate::Ray;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RayOp {
    d1: Vec4,
    d2: Vec4,
}

impl RayOp {
    const STAGE_KILLED: u32 = 0;
    const STAGE_PRIMARY: u32 = 1;
    const STAGE_REFLECTED: u32 = 2;

    fn new(origin: Vec3, direction: Vec3, stage: u32) -> Self {
        Self {
            d1: origin.extend(f32::from_bits(stage)),
            d2: direction.extend(f32::from_bits(0)),
        }
    }

    pub fn killed() -> Self {
        Self::new(Default::default(), Default::default(), Self::STAGE_KILLED)
    }

    pub fn primary(ray: Ray) -> Self {
        Self::new(ray.origin(), ray.direction(), Self::STAGE_PRIMARY)
    }

    pub fn reflected(ray: Ray) -> Self {
        Self::new(ray.origin(), ray.direction(), Self::STAGE_REFLECTED)
    }

    pub fn with_hit(mut self, instance_id: u32, triangle_id: u32) -> Self {
        self.d1.w = f32::from_bits((instance_id << 2) | self.stage());
        self.d2.w = f32::from_bits(triangle_id);
        self
    }

    pub fn ray(self) -> Ray {
        Ray::new(self.d1.xyz(), self.d2.xyz())
    }

    pub fn stage(self) -> u32 {
        self.d1.w.to_bits() & 0b11
    }

    pub fn is_killed(self) -> bool {
        self.stage() == Self::STAGE_KILLED
    }

    pub fn is_primary(self) -> bool {
        self.stage() == Self::STAGE_PRIMARY
    }

    pub fn is_reflected(self) -> bool {
        self.stage() == Self::STAGE_REFLECTED
    }

    pub fn instance_id(self) -> u32 {
        self.d1.w.to_bits() >> 2
    }

    pub fn triangle_id(self) -> u32 {
        self.d2.w.to_bits()
    }
}
