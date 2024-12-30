mod bilinear_filter;
mod f32_ext;
mod u32_ext;
mod vec2_ext;
mod vec3_ext;

use core::ops;

use glam::{uvec2, UVec2, Vec3};
use spirv_std::Image;

pub use self::bilinear_filter::*;
pub use self::f32_ext::*;
pub use self::u32_ext::*;
pub use self::vec2_ext::*;
pub use self::vec3_ext::*;

pub type Tex<'a> = &'a Image!(2D, type = f32, sampled);
pub type TexRgba8<'a> = &'a Image!(2D, format = rgba8, sampled = false);
pub type TexRgba16<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32<'a> = &'a Image!(2D, format = rgba32f, sampled = false);

pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: ops::Add<Output = T>,
    T: ops::Sub<Output = T>,
    T: ops::Mul<f32, Output = T>,
    T: Copy,
{
    a + (b - a) * t.clamp(0.0, 1.0)
}

pub fn resolve_checkerboard(global_id: UVec2, frame: u32) -> UVec2 {
    global_id * uvec2(2, 1) + uvec2((frame + global_id.y) % 2, 0)
}

pub fn resolve_checkerboard_alt(global_id: UVec2, frame: u32) -> UVec2 {
    resolve_checkerboard(global_id, frame + 1)
}

pub fn got_checkerboard_at(screen_pos: UVec2, frame: u32) -> bool {
    screen_pos == resolve_checkerboard(screen_pos / uvec2(2, 1), frame)
}

use spirv_std::num_traits::Float;

pub trait Vec3StrolleExt {
    fn safe_any_orthonormal_pair(&self) -> (Vec3, Vec3);
}

impl Vec3StrolleExt for Vec3 {
    fn safe_any_orthonormal_pair(&self) -> (Vec3, Vec3) {
        let sign = 1.0f32.copysign(self.z);
        let a = -1.0 / (sign + self.z);
        let b = self.x * self.y * a;

        (
            Vec3::new(
                1.0 + sign * self.x * self.x * a,
                sign * b,
                -sign * self.x,
            ),
            Vec3::new(b, sign + self.y * self.y * a, -self.y),
        )
    }
}

pub const STROLLE_EPSILON: f32 = 0.000_0001;
