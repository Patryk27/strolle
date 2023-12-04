mod bilinear_filter;
mod f32_ext;
mod u32_ext;
mod vec2_ext;
mod vec3_ext;

use core::ops;

use glam::{vec3, Vec3};
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

pub fn rgb_to_ycocg(val: Vec3) -> Vec3 {
    let co = val.x - val.z;
    let tmp = val.z + co / 2.0;
    let cg = val.y - tmp;
    let y = tmp + cg / 2.0;

    vec3(y, co, cg)
}

pub fn ycocg_to_rgb(val: Vec3) -> Vec3 {
    let tmp = val.x - val.z / 2.0;
    let g = val.z + tmp;
    let b = tmp - val.y / 2.0;
    let r = b + val.y;

    vec3(r, g, b)
}
