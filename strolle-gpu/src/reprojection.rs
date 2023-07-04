use glam::{vec2, vec4, UVec2, Vec4};

use crate::TexRgba32f;

#[derive(Clone, Copy, Default)]
pub struct Reprojection {
    pub prev_x: f32,
    pub prev_y: f32,
    pub confidence: f32,
}

impl Reprojection {
    pub fn serialize(&self) -> Vec4 {
        vec4(
            self.prev_x,
            self.prev_y,
            Default::default(),
            self.confidence,
        )
    }

    pub fn deserialize(d0: Vec4) -> Self {
        Self {
            prev_x: d0.x,
            prev_y: d0.y,
            confidence: d0.w,
        }
    }

    pub fn is_some(&self) -> bool {
        self.confidence > 0.0
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn prev_screen_pos(&self) -> UVec2 {
        vec2(self.prev_x, self.prev_y).round().as_uvec2()
    }
}

pub struct ReprojectionMap<'a> {
    tex: TexRgba32f<'a>,
}

impl<'a> ReprojectionMap<'a> {
    pub fn new(tex: TexRgba32f<'a>) -> Self {
        Self { tex }
    }

    pub fn get(&self, screen_pos: UVec2) -> Reprojection {
        Reprojection::deserialize(self.tex.read(screen_pos))
    }

    pub fn set(&self, screen_pos: UVec2, reprojection: &Reprojection) {
        unsafe {
            self.tex.write(screen_pos, reprojection.serialize());
        }
    }
}
