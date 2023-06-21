use core::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use glam::{vec3, Mat4, Vec3};
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

    pub fn sun_position(&self) -> Vec3 {
        // TODO we need this rotation because normal-remapping logic in the
        //      `Atmosphere` struct is invalid
        //
        // TODO recalculate the distance factor using sun's solid angle
        Mat4::from_axis_angle(Vec3::Y, -PI / 2.0)
            .transform_vector3(self.sun_direction() * 1000.0)
    }
}
