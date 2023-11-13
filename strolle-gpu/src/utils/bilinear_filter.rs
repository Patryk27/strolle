use glam::{ivec2, vec2, vec4, IVec2, UVec2, Vec2, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::Reprojection;

#[derive(Clone, Copy)]
pub struct BilinearFilter {
    /// Sample at `f(x=0, y=0)`
    pub s00: Vec4,

    /// Sample at `f(x=1, y=0)`
    pub s10: Vec4,

    /// Sample at `f(x=0, y=1)`
    pub s01: Vec4,

    /// Sample at `f(x=1, y=1)`
    pub s11: Vec4,

    /// Weights for each sample
    pub weights: Vec4,
}

impl BilinearFilter {
    pub fn reproject(
        reprojection: Reprojection,
        sample: impl Fn(UVec2) -> (Vec4, f32),
    ) -> Vec4 {
        if reprojection.is_exact() {
            sample(reprojection.prev_pos_round()).0
        } else {
            Self::from_reprojection(reprojection, sample).eval(vec2(
                reprojection.prev_x.fract(),
                reprojection.prev_y.fract(),
            ))
        }
    }

    pub fn from_reprojection(
        reprojection: Reprojection,
        sample: impl Fn(UVec2) -> (Vec4, f32),
    ) -> Self {
        let mut s00 = Vec4::ZERO;
        let mut s10 = Vec4::ZERO;
        let mut s01 = Vec4::ZERO;
        let mut s11 = Vec4::ZERO;
        let mut weights = Vec4::ZERO;

        let [p00, p10, p01, p11] =
            Self::reprojection_coords(reprojection.prev_x, reprojection.prev_y);

        if reprojection.validity & 0b0001 > 0 && p00.x > 0 && p00.y > 0 {
            (s00, weights.x) = sample(p00.as_uvec2());
        }

        if reprojection.validity & 0b0010 > 0 && p10.x > 0 && p10.y > 0 {
            (s10, weights.y) = sample(p10.as_uvec2());
        }

        if reprojection.validity & 0b0100 > 0 && p01.x > 0 && p01.y > 0 {
            (s01, weights.z) = sample(p01.as_uvec2());
        }

        if reprojection.validity & 0b1000 > 0 && p11.x > 0 && p11.y > 0 {
            (s11, weights.w) = sample(p11.as_uvec2());
        }

        Self {
            s00,
            s10,
            s01,
            s11,
            weights,
        }
    }

    pub fn reprojection_coords(prev_x: f32, prev_y: f32) -> [IVec2; 4] {
        let p00 = ivec2(prev_x.floor() as i32, prev_y.floor() as i32);
        let p10 = ivec2(prev_x.ceil() as i32, prev_y.floor() as i32);
        let p01 = ivec2(prev_x.floor() as i32, prev_y.ceil() as i32);
        let p11 = ivec2(prev_x.ceil() as i32, prev_y.ceil() as i32);

        [p00, p10, p01, p11]
    }

    pub fn eval(&self, uv: Vec2) -> Vec4 {
        let weights = self.weights
            * vec4(
                (1.0 - uv.x) * (1.0 - uv.y),
                uv.x * (1.0 - uv.y),
                (1.0 - uv.x) * uv.y,
                uv.x * uv.y,
            );

        let w_sum = weights.dot(Vec4::ONE);

        if w_sum == 0.0 {
            Default::default()
        } else {
            (self.s00 * weights.x
                + self.s10 * weights.y
                + self.s01 * weights.z
                + self.s11 * weights.w)
                / w_sum
        }
    }
}
