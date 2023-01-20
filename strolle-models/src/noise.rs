use core::f32::consts::PI;

use glam::{vec3, Vec3};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

#[derive(Copy, Clone)]
pub struct Noise {
    seed: u32,
}

impl Noise {
    pub fn new(seed: u32, global_idx: usize) -> Self {
        Self {
            seed: seed + (global_idx as u32),
        }
    }

    pub fn sample(&mut self) -> f32 {
        self.seed = Self::hash(self.seed);

        (self.seed as f32) / (u32::MAX as f32)
    }

    pub fn sample_sphere(&mut self) -> Vec3 {
        let theta = 2.0 * PI * self.sample();
        let phi = PI * self.sample();

        vec3(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos())
    }

    pub fn sample_hemisphere(&mut self) -> Vec3 {
        let z = self.sample() * 2.0 - 1.0;
        let a = self.sample() * 2.0 * PI;
        let r = (1.0 - z * z).sqrt();
        let x = r * a.cos();
        let y = r * a.sin();

        vec3(x, y, z)
    }

    fn hash(mut n: u32) -> u32 {
        n = (n ^ 61) ^ (n >> 16);
        n *= 9;
        n = n ^ (n >> 4);
        n *= 0x27d4eb2d;
        n = n ^ (n >> 15);
        n
    }
}
