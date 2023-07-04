mod bilinear_filter;
mod temporal_denoiser;

use spirv_std::Image;

pub use self::bilinear_filter::*;
pub use self::temporal_denoiser::*;

pub type TexRgba16f<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32f<'a> = &'a Image!(2D, format = rgba32f, sampled = false);
