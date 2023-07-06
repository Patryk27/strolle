mod bilinear_filter;
mod f32_ext;
mod temporal_denoiser;
mod vec3_ext;

use spirv_std::Image;

pub use self::bilinear_filter::*;
pub use self::f32_ext::*;
pub use self::temporal_denoiser::*;
pub use self::vec3_ext::*;

pub type TexRgba8f<'a> = &'a Image!(2D, format = rgba8, sampled = false);
pub type TexRgba16f<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32f<'a> = &'a Image!(2D, format = rgba32f, sampled = false);
