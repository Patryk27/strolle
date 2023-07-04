use glam::{ivec2, vec2, IVec2, Vec2, Vec4};
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
}

impl BilinearFilter {
    pub fn from_reprojection(
        reprojection: Reprojection,
        sample: impl Fn(IVec2) -> Vec4,
    ) -> Self {
        Self {
            s00: sample(ivec2(
                reprojection.prev_x.floor() as i32,
                reprojection.prev_y.floor() as i32,
            )),
            s10: sample(ivec2(
                reprojection.prev_x.ceil() as i32,
                reprojection.prev_y.floor() as i32,
            )),
            s01: sample(ivec2(
                reprojection.prev_x.floor() as i32,
                reprojection.prev_y.ceil() as i32,
            )),
            s11: sample(ivec2(
                reprojection.prev_x.ceil() as i32,
                reprojection.prev_y.ceil() as i32,
            )),
        }
    }

    pub fn eval(&self, uv: Vec2) -> Vec4 {
        let s00 = self.s00 * (1.0 - uv.x) * (1.0 - uv.y);
        let s10 = self.s10 * uv.x * (1.0 - uv.y);
        let s01 = self.s01 * (1.0 - uv.x) * uv.y;
        let s11 = self.s11 * uv.x * uv.y;

        s00 + s10 + s01 + s11
    }

    pub fn eval_reprojection(&self, reprojection: Reprojection) -> Vec4 {
        self.eval(vec2(
            reprojection.prev_x.fract(),
            reprojection.prev_y.fract(),
        ))
    }
}
