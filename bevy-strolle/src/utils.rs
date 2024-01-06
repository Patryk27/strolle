use bevy::math::{vec3, vec4};
use bevy::prelude::*;

pub fn color_to_vec3(color: Color) -> Vec3 {
    let [r, g, b, _] = color.as_linear_rgba_f32();

    vec3(r, g, b)
}

pub fn color_to_vec4(color: Color) -> Vec4 {
    let [r, g, b, a] = color.as_linear_rgba_f32();

    vec4(r, g, b, a)
}
