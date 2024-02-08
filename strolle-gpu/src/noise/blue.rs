use core::f32::consts::PI;

use glam::{uvec2, vec3, Mat3, UVec2, Vec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{F32Ext, TexRgba8};

pub struct BlueNoise<'a> {
    tex: TexRgba8<'a>,
    uv: UVec2,
}

impl<'a> BlueNoise<'a> {
    pub const SIZE: UVec2 = uvec2(256, 256);

    pub fn new(tex: TexRgba8<'a>, id: UVec2, frame: u32) -> Self {
        let uv = (id + uvec2(71, 11) * frame) % Self::SIZE;

        Self { tex, uv }
    }

    pub fn first_sample(&self) -> Vec2 {
        self.tex.read(self.uv).xy()
    }

    pub fn second_sample(&self) -> Vec2 {
        self.tex.read(self.uv).zw()
    }

    pub fn sample_hemisphere(&self, normal: Vec3) -> Vec3 {
        let sample = self.second_sample();

        let matrix = {
            let (t, b) = normal.any_orthonormal_pair();

            Mat3 {
                x_axis: t,
                y_axis: b,
                z_axis: normal,
            }
        };

        let sample = {
            let cos_theta = sample.x;
            let sin_theta = (1.0f32 - cos_theta.sqr()).sqrt();
            let phi = 2.0 * PI * sample.y;

            vec3(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
        };

        matrix * sample
    }
}

pub struct LdsBlueNoise<'a> {
    sobol: &'a [u32],
    scrambling_tile: &'a [u32],
    ranking_tile: &'a [u32],
    pixel_i: u32,
    pixel_j: u32,
    sample_index: u32,
    sample_dimension: u32,
}

impl<'a> LdsBlueNoise<'a> {
    pub fn new(
        sobol: &'a [u32],
        scrambling_tile: &'a [u32],
        ranking_tile: &'a [u32],
        id: UVec2,
        frame: u32,
        dimension: u32,
    ) -> Self {
        Self {
            sobol,
            scrambling_tile,
            ranking_tile,
            pixel_i: id.x,
            pixel_j: id.y,
            sample_index: frame,
            sample_dimension: dimension,
        }
    }

    pub fn sample(&mut self) -> f32 {
        let pixel_i = self.pixel_i & 127;
        let pixel_j = self.pixel_j & 127;
        let sample_index = self.sample_index & 255;
        let sample_dimension = self.sample_dimension & 255;

        let ranked_sample_index = sample_index
            ^ self.ranking_tile
                [(sample_dimension + (pixel_i + pixel_j * 128) * 8) as usize];

        let value =
            self.sobol[(sample_dimension + ranked_sample_index * 256) as usize];

        let value = value
            ^ self.scrambling_tile[((sample_dimension % 8)
                + (pixel_i + pixel_j * 128) * 8)
                as usize];

        self.sample_dimension += 1;

        (0.5 + (value as f32)) / 256.0
    }

    pub fn sample_hemisphere(&mut self, normal: Vec3) -> Vec3 {
        let matrix = {
            let (t, b) = normal.any_orthonormal_pair();

            Mat3 {
                x_axis: t,
                y_axis: b,
                z_axis: normal,
            }
        };

        let sample = {
            let cos_theta = self.sample();
            let sin_theta = (1.0f32 - cos_theta.sqr()).sqrt();
            let phi = 2.0 * PI * self.sample();

            vec3(phi.cos() * sin_theta, phi.sin() * sin_theta, cos_theta)
        };

        matrix * sample
    }
}
