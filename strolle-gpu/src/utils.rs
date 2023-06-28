use glam::{uvec2, UVec2, Vec3};
use spirv_std::Image;

pub type TexRgba16f<'a> = &'a Image!(2D, format = rgba16f, sampled = false);
pub type TexRgba32f<'a> = &'a Image!(2D, format = rgba32f, sampled = false);

/// Upgrades coordinates from half-viewport to full-viewport using per-frame
/// scrambling.
pub fn upsample(id: UVec2, frame: u32) -> UVec2 {
    let subpixels = [uvec2(1, 1), uvec2(1, 0), uvec2(0, 0), uvec2(0, 1)];

    2 * id + subpixels[(frame % 4) as usize]
}

pub trait Vec3Ext {
    /// Clips given value (presumably a color) into given bounding box.
    ///
    /// https://s3.amazonaws.com/arena-attachments/655504/c5c71c5507f0f8bf344252958254fb7d.pdf?1468341463
    fn clip(&self, aabb_min: Vec3, aabb_max: Vec3) -> Vec3;
}

impl Vec3Ext for Vec3 {
    fn clip(&self, aabb_min: Vec3, aabb_max: Vec3) -> Vec3 {
        let p_clip = 0.5 * (aabb_max + aabb_min);
        let e_clip = 0.5 * (aabb_max - aabb_min);
        let v_clip = *self - p_clip;
        let v_unit = v_clip / e_clip;
        let a_unit = v_unit.abs();
        let ma_unit = a_unit.max_element();

        if ma_unit > 1.0 {
            p_clip + v_clip / ma_unit
        } else {
            *self
        }
    }
}
