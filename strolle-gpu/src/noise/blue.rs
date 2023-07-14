use core::f32::consts::PI;

use glam::{uvec2, vec3, UVec2, Vec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::TexRgba8f;

pub struct BlueNoise<'a> {
    tex: TexRgba8f<'a>,
    uv: UVec2,
}

impl<'a> BlueNoise<'a> {
    pub const SIZE: UVec2 = uvec2(256, 256);

    pub fn new(tex: TexRgba8f<'a>, id: UVec2, frame: u32) -> Self {
        let uv = (id + uvec2(71, 11) * frame) % Self::SIZE;

        Self { tex, uv }
    }

    pub fn first_sample(self) -> Vec2 {
        self.tex.read(self.uv).xy()
    }

    pub fn second_sample(self) -> Vec2 {
        self.tex.read(self.uv).zw()
    }

    pub fn sample_hemisphere(self, normal: Vec3) -> Vec3 {
        let u = self.second_sample();

        let radius = (1.0f32 - u.x * u.x).sqrt();
        let angle = 2.0 * PI * u.y;

        let b = normal.cross(vec3(0.0, 1.0, 1.0)).normalize();
        let t = b.cross(normal);

        (radius * angle.sin() * b + u.x * normal + radius * angle.cos() * t)
            .normalize()
    }
}
