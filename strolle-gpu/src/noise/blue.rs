use glam::{uvec2, UVec2, Vec2, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Frame, TexRgba8};

pub struct BlueNoise<'a> {
    tex: TexRgba8<'a>,
    uv: UVec2,
}

impl<'a> BlueNoise<'a> {
    pub const SIZE: UVec2 = uvec2(256, 256);

    pub fn new(tex: TexRgba8<'a>, id: UVec2, frame: Frame) -> Self {
        let uv = (id + uvec2(71, 11) * frame.get()) % Self::SIZE;

        Self { tex, uv }
    }

    pub fn first_sample(self) -> Vec2 {
        self.tex.read(self.uv).xy()
    }

    pub fn second_sample(self) -> Vec2 {
        self.tex.read(self.uv).zw()
    }
}
