use glam::{vec2, vec4, UVec2, Vec2, Vec4};

use crate::TexRgba32;

#[derive(Clone, Copy, Default)]
pub struct Reprojection {
    pub prev_x: f32,
    pub prev_y: f32,
    pub confidence: f32,
    pub validity: u32,
}

impl Reprojection {
    pub fn serialize(&self) -> Vec4 {
        vec4(
            self.prev_x,
            self.prev_y,
            self.confidence,
            f32::from_bits(self.validity),
        )
    }

    pub fn deserialize(d0: Vec4) -> Self {
        Self {
            prev_x: d0.x,
            prev_y: d0.y,
            confidence: d0.z,
            validity: d0.w.to_bits(),
        }
    }

    pub fn is_some(&self) -> bool {
        self.confidence > 0.0
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn prev_pos(&self) -> Vec2 {
        vec2(self.prev_x, self.prev_y)
    }

    pub fn prev_pos_round(&self) -> UVec2 {
        self.prev_pos().round().as_uvec2()
    }

    pub fn prev_pos_fract(&self) -> Vec2 {
        self.prev_pos().fract()
    }

    pub fn is_exact(&self) -> bool {
        self.prev_pos_fract().length_squared() == 0.0
    }
}

pub struct ReprojectionMap<'a> {
    tex: TexRgba32<'a>,
}

impl<'a> ReprojectionMap<'a> {
    pub fn new(tex: TexRgba32<'a>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let target = Reprojection {
            prev_x: 123.45,
            prev_y: 234.56,
            confidence: 1.23,
            validity: 0xcafebabe,
        };

        let target = Reprojection::deserialize(target.serialize());

        assert_eq!(123.45, target.prev_x);
        assert_eq!(234.56, target.prev_y);
        assert_eq!(1.23, target.confidence);
        assert_eq!(0xcafebabe, target.validity);
    }
}
