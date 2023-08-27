mod bilinear_filter;
mod f32_ext;
mod temporal_denoiser;
mod u32_ext;
mod vec3_ext;

use core::ops;

use spirv_std::Image;

pub use self::bilinear_filter::*;
pub use self::f32_ext::*;
pub use self::temporal_denoiser::*;
pub use self::u32_ext::*;
pub use self::vec3_ext::*;

pub type Tex<'a> = &'a Image!(2D, type = f32, sampled);
pub type TexRgba8f<'a> = &'a Image!(2D, format = rgba8, sampled = false);
pub type TexRgba16f<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32f<'a> = &'a Image!(2D, format = rgba32f, sampled = false);

// TODO make a trait
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: ops::Add<Output = T>,
    T: ops::Sub<Output = T>,
    T: ops::Mul<f32, Output = T>,
    T: Copy,
{
    a + (b - a) * t.clamp(0.0, 1.0)
}
