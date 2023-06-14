use glam::{UVec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::TexRgba32f;

#[derive(Clone, Copy)]
pub struct Geometry {
    pub normal: Vec3,
    pub depth: f32,
}

impl Geometry {
    pub fn evaluate_similarity_to(&self, other: &Self) -> f32 {
        let normal = self.normal.dot(other.normal).max(0.0);

        let depth = {
            let depth = (self.depth - other.depth).abs();

            if depth >= 1.0 {
                0.0
            } else {
                1.0 - depth
            }
        };

        normal * depth
    }
}

pub struct GeometryMap<'a> {
    tex: TexRgba32f<'a>,
}

impl<'a> GeometryMap<'a> {
    pub fn new(tex: TexRgba32f<'a>) -> Self {
        Self { tex }
    }

    pub fn get(&self, screen_pos: UVec2) -> Geometry {
        let d0 = self.tex.read(screen_pos);

        Geometry {
            normal: d0.xyz(),
            depth: d0.w,
        }
    }
}
