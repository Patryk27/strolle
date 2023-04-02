use std::collections::HashMap;
use std::f32::consts::PI;

use glam::{vec3, Quat, UVec2, Vec3, Vec3Swizzles};
use image::{ImageBuffer, Rgb};

const RESOLUTION: usize = 1024;

fn main() {
    let mut samples: HashMap<_, usize> = Default::default();
    let mut noise = Noise::new(123, 234, 345);

    for _ in 0..10_000_000 {
        let sample = noise.sample_hemisphere(vec3(0.0, 0.0, 1.0));

        assert!(
            sample.length() >= 0.99 && sample.length() <= 1.01,
            "Sample has invalid length: {sample:?} (={})",
            sample.length()
        );

        let sample = Vec3::splat(RESOLUTION as f32 / 2.0)
            + sample * (RESOLUTION as f32 / 2.0);

        assert!(
            sample.x >= 0.0
                && sample.y >= 0.0
                && sample.z >= 0.0
                && sample.x <= 1024.0
                && sample.y <= 1024.0
                && sample.z <= 1024.0,
            "Sample out of range: {sample:?}"
        );

        let sample = sample
            .xy()
            .as_uvec2()
            .min(UVec2::splat((RESOLUTION - 1) as _));

        // let sample = noise.sample_sphere().xy();
        // let sample = sample * (RESOLUTION as f32) * 0.5;
        // let sample =
        //     sample + Vec2::splat((RESOLUTION as f32) * 0.5);
        // let sample = sample.as_uvec2();

        *samples.entry(sample).or_default() += 1;
    }

    println!("unique samples: {}", samples.len());

    // ---

    let mut image =
        ImageBuffer::<Rgb<u8>, _>::new(RESOLUTION as _, RESOLUTION as _);

    let max_sample_count = *samples.values().max().unwrap();

    for (sample, sample_count) in &samples {
        let color = (255 * *sample_count / max_sample_count) as u8;

        image.get_pixel_mut(sample.x, sample.y).0 = [color, color, color];
    }

    image.save("output.png").unwrap();
}

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
