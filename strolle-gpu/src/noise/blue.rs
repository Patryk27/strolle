use glam::{uvec2, UVec2, Vec2, Vec4Swizzles};

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
}
