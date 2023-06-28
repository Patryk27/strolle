use bevy::math::{vec3, vec4, Affine3A, Mat3A, Vec3A};
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
        st::glam::UVec2::new(self.x, self.y)
    }
}

impl GlamCompat<st::glam::Vec3> for Vec3 {
    fn compat(self) -> st::glam::Vec3 {
        st::glam::Vec3::new(self.x, self.y, self.z)
    }
}

impl GlamCompat<st::glam::Vec3A> for Vec3A {
    fn compat(self) -> st::glam::Vec3A {
        st::glam::Vec3A::new(self.x, self.y, self.z)
    }
}

impl GlamCompat<st::glam::Vec4> for Vec4 {
    fn compat(self) -> st::glam::Vec4 {
        st::glam::Vec4::new(self.x, self.y, self.z, self.w)
    }
}

impl GlamCompat<st::glam::Affine3A> for Affine3A {
    fn compat(self) -> st::glam::Affine3A {
        st::glam::Affine3A {
            matrix3: self.matrix3.compat(),
            translation: self.translation.compat(),
        }
    }
}

impl GlamCompat<st::glam::Mat3A> for Mat3A {
    fn compat(self) -> st::glam::Mat3A {
        st::glam::Mat3A {
            x_axis: self.x_axis.compat(),
            y_axis: self.y_axis.compat(),
            z_axis: self.z_axis.compat(),
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
