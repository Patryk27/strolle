use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct World {
    pub light_count: u32,
    pub sun_azimuth: f32,
    pub sun_altitude: f32,
}

impl World {
    // TODO recalculate the distance factor using sun's solid angle
    pub const SUN_DISTANCE: f32 = 1000.0;

    pub fn sun_direction(&self) -> Vec3 {
        vec3(
            self.sun_altitude.cos() * self.sun_azimuth.sin(),
            self.sun_altitude.sin(),
            -self.sun_altitude.cos() * self.sun_azimuth.cos(),
        )
    }

    pub fn sun_position(&self) -> Vec3 {
        self.sun_direction() * Self::SUN_DISTANCE
    }
}
