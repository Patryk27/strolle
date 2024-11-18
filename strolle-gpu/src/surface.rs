use glam::{UVec2, Vec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Normal, TexRgba32};

#[derive(Clone, Copy)]
pub struct Surface {
    pub normal: Vec3,
    pub depth: f32,
    pub roughness: f32,
}

impl Surface {
    pub fn is_sky(self) -> bool {
        self.depth == 0.0
    }

    /// Returns a score `<0.0, 1.0>` that determines the similarity of two given
    /// surfaces.
    pub fn evaluate_similarity_to(self, other: Self) -> f32 {
        if self.is_sky() || other.is_sky() {
            return 0.0;
        }

        let normal_score = {
            let dot = self.normal.dot(other.normal).max(0.0);

            if dot <= 0.5 {
                0.0
            } else {
                2.0 * dot
            }
        };

        let depth_score = {
            let t = (self.depth - other.depth).abs();

            if t >= 0.1 * other.depth {
                0.0
            } else {
                1.0
            }
        };

        normal_score * depth_score
    }
}

#[derive(Clone, Copy)]
pub struct SurfaceMap<'a> {
    tex: TexRgba32<'a>,
}

impl<'a> SurfaceMap<'a> {
    pub fn new(tex: TexRgba32<'a>) -> Self {
        Self { tex }
    }

    pub fn get(self, screen_pos: UVec2) -> Surface {
        let d0 = self.tex.read(screen_pos);

        let depth = d0.x;
        let normal = Normal::decode(Vec2::new(d0.y, d0.z));

        let packed = d0.w.to_bits();
        let roughness_u = (packed >> 16) & 0xFF;
        let roughness = roughness_u as f32 / 255.0;

        Surface {
            normal: normal,
            depth: depth,
            roughness: roughness,
        }
    }
}


