use bevy::math::{vec3, vec4};
use bevy::prelude::*;
use strolle as st;

pub fn color_to_vec3(color: Color) -> Vec3 {
    let [r, g, b, _] = color.as_linear_rgba_f32();

    vec3(r, g, b)
}

pub fn color_to_vec4(color: Color) -> Vec4 {
    let [r, g, b, a] = color.as_linear_rgba_f32();

    vec4(r, g, b, a)
}

/// Compatibility layer for different versions of `glam`.
///
/// This comes handy because sometimes Bevy and rust-gpu use different versions
/// of glam which - without this compatibility layer - would throw a compilation
/// error.
pub trait GlamCompat<T> {
    fn compat(self) -> T;
}

impl GlamCompat<st::glam::UVec2> for UVec2 {
    fn compat(self) -> st::glam::UVec2 {
        st::glam::UVec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl GlamCompat<st::glam::Vec3> for Vec3 {
    fn compat(self) -> st::glam::Vec3 {
        st::glam::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl GlamCompat<st::glam::Vec4> for Vec4 {
    fn compat(self) -> st::glam::Vec4 {
        st::glam::Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: self.w,
        }
    }
}

impl GlamCompat<st::glam::Mat4> for Mat4 {
    fn compat(self) -> st::glam::Mat4 {
        st::glam::Mat4 {
            x_axis: self.x_axis.compat(),
            y_axis: self.y_axis.compat(),
            z_axis: self.z_axis.compat(),
            w_axis: self.w_axis.compat(),
        }
    }
}
