use glam::Vec2;

pub trait Vec2Ext
where
    Self: Sized,
{
    /// Clips this color-vector into given bounding box.
    ///
    /// See:
    /// - https://s3.amazonaws.com/arena-attachments/655504/c5c71c5507f0f8bf344252958254fb7d.pdf?1468341463
    fn clip(self, aabb_min: Self, aabb_max: Self) -> Self;
}

impl Vec2Ext for Vec2 {
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
}
