use crate::*;

pub fn identity() -> Mat4 {
    Mat4::IDENTITY
}

pub fn translated(v: Vec3) -> Mat4 {
    Mat4::from_translation(v)
}

/// Translates the transformation matrix by the given translation vector.
pub fn translate(xform: &mut Mat4, v: Vec3) -> &mut Mat4 {
    *xform *= Mat4::from_translation(v);
    xform
}

pub fn set_translation(xform: &mut Mat4, v: Vec3) -> &mut Mat4 {
    xform.w_axis = v.extend(1.0);
    xform
}

/// Scales the transformation matrix along axes given by the vector.
pub fn scale(xform: &mut Mat4, v: Vec3) -> &mut Mat4 {
    *xform *= Mat4::from_scale(v);
    xform
}

/// Rotates the transation matrix by the given angle (in degrees) around the given axis.
pub fn rotate(xform: &mut Mat4, angle: f32, axis: Vec3) -> &mut Mat4 {
    *xform *= Mat4::from_axis_angle(axis, angle);
    xform
}

/// Applies the transformation matrix to the point.
pub fn transform(v: Vec3, xform: Mat4) -> Vec3 {
    let v = xform * v.extend(1.0);
    Vec3::new(v.x, v.y, v.z)
}
