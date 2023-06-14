use core::f32::consts::PI;

use glam::{vec3, UVec2, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[derive(Copy, Clone)]
pub struct Noise {
    state: u32,
}

impl Noise {
    pub fn new(seed: u32, id: UVec2) -> Self {
        Self {
            state: seed ^ (48619 * id.x) ^ (95461 * id.y),
        }
    }

    /// Generates a uniform sample in range (0.0, 1.0).
    pub fn sample(&mut self) -> f32 {
        (self.sample_int() as f32) / (u32::MAX as f32)
    }

    /// Generates a uniform sample in range (0, u32::MAX).
    pub fn sample_int(&mut self) -> u32 {
        self.state = self.state * 747796405 + 2891336453;

        let word =
            ((self.state >> ((self.state >> 28) + 4)) ^ self.state) * 277803737;

        (word >> 22) ^ word
    }

    /// Generates a uniform sample on a sphere.
    pub fn sample_sphere(&mut self) -> Vec3 {
        vec3(self.sample(), self.sample(), self.sample())
    }

    /// Generates a uniform sample on a hemisphere around given normal.
    pub fn sample_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        let u = glam::vec2(self.sample(), self.sample());

        let radius = (1.0f32 - u.x * u.x).sqrt();
        let angle = 2.0 * PI * u.y;

        let b = normal.cross(vec3(0.0, 1.0, 1.0)).normalize();
        let t = b.cross(normal);

        (radius * angle.sin() * b + u.x * normal + radius * angle.cos() * t)
            .normalize()
    }
}
