use glam::{UVec2, Vec3, Vec3Swizzles, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Normal, TexRgba32f};

#[derive(Clone, Copy)]
pub struct Surface {
    pub normal: Vec3,
    pub depth: f32,
}

impl Surface {
    /// Returns a score `<0.0, 1.0>` that determines the similarity of two given
    /// surfaces.
    ///
    /// See also: [`SurfaceMap::evaulate_similarity_between()`].
    pub fn evaluate_similarity_to(&self, other: &Self) -> f32 {
        let normal_score = self.normal.dot(other.normal).max(0.0);

        // TODO a continuous function here would be much, much better
        let depth_score = if self.depth < 35.0 && other.depth < 35.0 {
            1.0 - (self.depth - other.depth).abs().min(1.0)
        } else {
            1.0 - (self.depth.log2() - other.depth.log2()).abs().min(1.0)
        };

        normal_score * depth_score
    }
}

#[derive(Clone, Copy)]
pub struct SurfaceMap<'a> {
    tex: TexRgba32f<'a>,
}

impl<'a> SurfaceMap<'a> {
    pub fn new(tex: TexRgba32f<'a>) -> Self {
        Self { tex }
    }

    pub fn get(&self, screen_pos: UVec2) -> Surface {
        let d0 = self.tex.read(screen_pos);

        Surface {
            normal: Normal::decode(d0.xy()),
            depth: d0.z,
        }
    }

    /// Returns a score `<0.0, 1.0>` that determines the similarity of two given
    /// surfaces, including possible surfaces between them.
    ///
    /// This function performs screen-space ray-marching and so it is able to
    /// find discontinuities between surfaces etc.
    ///
    /// See also: [`Surface::evaluate_similarity_to()`].
    pub fn evaluate_similarity_between(
        &self,
        lhs: UVec2,
        lhs_surface: Surface,
        rhs: UVec2,
    ) -> f32 {
        let steps = lhs.as_vec2().distance(rhs.as_vec2()) / 3.0;
        let steps = steps.min(4.0) as i32;

        if steps == 0 {
            return 1.0;
        }

        let rhs_surface = self.get(rhs);

        if steps == 1 {
            return lhs_surface.evaluate_similarity_to(&rhs_surface);
        }

        let lhs = lhs.as_vec2().extend(lhs_surface.depth);
        let rhs = rhs.as_vec2().extend(rhs_surface.depth);
        let step = (rhs - lhs) / (steps as f32);

        let mut cursor = lhs;
        let mut step_idx = 0;
        let mut score = 1.0;

        while step_idx < steps {
            cursor += step;

            let cursor_surface = self.get(cursor.xy().as_uvec2());
            let depth_diff = (cursor_surface.depth - cursor.z).abs();

            if depth_diff >= 1.0 {
                return 0.0;
            }

            score *= 1.0 - depth_diff;
            score *= lhs_surface.normal.dot(cursor_surface.normal).max(0.0);
            step_idx += 1;
        }

        score
    }
}
