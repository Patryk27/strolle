use core::f32::consts::PI;

use glam::{vec3, Quat, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[derive(Copy, Clone)]
pub struct Noise {
    state: u32,
}

impl Noise {
    pub fn new(seed: u32, x: u32, y: u32) -> Self {
        Self {
            state: seed ^ (48619 * x) ^ (95461 * y),
        }
    }

    pub fn sample(&mut self) -> f32 {
        (self.sample_int() as f32) / (u32::MAX as f32)
    }

    pub fn sample_int(&mut self) -> u32 {
        let state = self.state;
        self.state = self.state * 747796405 + 2891336453;
        let word = ((state >> ((state >> 28) + 4)) ^ state) * 277803737;

        (word >> 22) ^ word
    }

    pub fn sample_sphere(&mut self) -> Vec3 {
        vec3(self.sample(), self.sample(), self.sample())
    }

    pub fn sample_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        fn align(sample: Vec3, up: Vec3, normal: Vec3) -> Vec3 {
            let angle = up.dot(normal).acos();
            let axis = up.cross(normal);

            Quat::from_axis_angle(axis, angle) * sample
        }

        align(self.sample_hemisphere_inner(), vec3(0.0, 0.0, 1.0), normal)
    }

    fn sample_hemisphere_inner(&mut self) -> Vec3 {
        let u = self.sample();
        let v = self.sample();
        let m = 2.5;

        let theta = (1.0 - u).powf(1.0 / (1.0 + m)).acos();
        let phi = 2.0 * PI * v;

        let x = theta.sin() * phi.cos();
        let y = theta.sin() * phi.sin();

        vec3(x, y, (1.0 - x * x - y * y).sqrt())
    }
}
