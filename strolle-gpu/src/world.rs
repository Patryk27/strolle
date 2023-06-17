use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[repr(C)]
#[derive(Copy, Clone, Default, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct World {
    pub light_count: u32,
    pub sun_altitude: f32,
}

impl World {
    pub fn sun_direction(&self) -> Vec3 {
        vec3(0.0, self.sun_altitude.sin(), -self.sun_altitude.cos())
    }
}
