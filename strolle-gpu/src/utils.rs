use glam::{uvec2, UVec2};
use spirv_std::Image;

pub type TexRgba16f<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32f<'a> = &'a Image!(2D, format = rgba32f, sampled = false);

/// Upgrades coordinates from half-viewport to full-viewport using per-frame
/// scrambling.
pub fn upsample(id: UVec2, frame: u32) -> UVec2 {
    let subpixels = [uvec2(1, 1), uvec2(1, 0), uvec2(0, 0), uvec2(0, 1)];

    2 * id + subpixels[(frame % 4) as usize]
}
