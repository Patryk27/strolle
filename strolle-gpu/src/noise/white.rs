use core::f32::consts::PI;

use glam::{vec2, vec3, UVec2, Vec2, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Vec3StrolleExt, F32Ext};

#[derive(Clone, Copy)]
pub struct WhiteNoise {
    state: u32,
}

impl WhiteNoise {
    pub fn new(seed: u32, id: UVec2) -> Self {
        Self {
            state: seed ^ (48619 * id.x) ^ (95461 * id.y),
        }
    }

    pub fn from_state(state: u32) -> Self {
        Self { state }
    }

    pub fn state(self) -> u32 {
        self.state
    }

    /// Generates a uniform sample in range `<0.0, 1.0>`.
    pub fn sample(&mut self) -> f32 {
        (self.sample_int() as f32) / (u32::MAX as f32)
    }

    /// Generates a uniform sample in range `<0, u32::MAX>`.
    pub fn sample_int(&mut self) -> u32 {
        self.state = self.state * 747796405 + 2891336453;

        let word =
            ((self.state >> ((self.state >> 28) + 4)) ^ self.state) * 277803737;

        (word >> 22) ^ word
    }

    /// Generates a uniform sample on a circle.
    pub fn sample_circle(&mut self) -> Vec2 {
        let angle = self.sample() * PI * 2.0;

        vec2(angle.cos(), angle.sin())
    }

    /// Generates a uniform sample inside of a disk.
    pub fn sample_disk(&mut self) -> Vec2 {
        let radius = self.sample().sqrt();

        self.sample_circle() * radius
    }

    /// Generates a uniform sample on a sphere.
    pub fn sample_sphere(&mut self) -> Vec3 {
        let phi = self.sample() * 2.0 * PI;
        let cos_theta = self.sample() * 2.0 - 1.0;
        let u = self.sample();

        let theta = cos_theta.acos();
        let r = u.sqrt();

        vec3(
            r * theta.sin() * phi.cos(),
            r * theta.sin() * phi.sin(),
            r * theta.cos(),
        )
    }

    /// Generates a uniform sample on a hemisphere around given normal.
    pub fn sample_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        let cos_theta = self.sample();
        let sin_theta = (1.0f32 - cos_theta.sqr()).max(0.0).sqrt();
        let phi = 2.0 * PI * self.sample();
        let (t, b) = normal.safe_any_orthonormal_pair();

        (t * phi.cos() + b * phi.sin()) * sin_theta + normal * cos_theta
    }
}
