use glam::{vec3, Vec3};

pub trait Vec3Ext
where
    Self: Sized,
{
    /// Reflects this direction-vector around `other`.
    fn reflect(self, other: Self) -> Self;

    /// Clips this color-vector into given bounding box.
    ///
    /// See:
    /// - https://s3.amazonaws.com/arena-attachments/655504/c5c71c5507f0f8bf344252958254fb7d.pdf?1468341463
    fn clip(self, aabb_min: Self, aabb_max: Self) -> Self;

    /// Returns luminance of this color-vector.
    fn luminance(self) -> f32;

    /// Adjusts luminance of this color-vector.
    fn with_luminance(self, l_out: f32) -> Self;
}

impl Vec3Ext for Vec3 {
    fn reflect(self, other: Self) -> Self {
        self - 2.0 * other.dot(self) * other
    }

    fn clip(self, aabb_min: Self, aabb_max: Self) -> Self {
        let p_clip = 0.5 * (aabb_max + aabb_min);
        let e_clip = 0.5 * (aabb_max - aabb_min);
        let v_clip = self - p_clip;
        let v_unit = v_clip / e_clip;
        let a_unit = v_unit.abs();
        let ma_unit = a_unit.max_element();

        if ma_unit > 1.0 {
            p_clip + v_clip / ma_unit
        } else {
            self
        }
    }

    fn luminance(self) -> f32 {
        self.dot(vec3(0.2126, 0.7152, 0.0722))
    }

    fn with_luminance(self, l_out: f32) -> Self {
        self * (l_out / self.luminance())
    }
}
