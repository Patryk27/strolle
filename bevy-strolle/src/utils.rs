
use bevy::prelude::*;

pub fn color_to_vec3(color: Color) -> Vec3 {
    let [r, g, b, _] = color.to_linear().to_f32_array();

    Vec3::new(r, g, b)
}

pub fn color_to_vec4(color: Color) -> Vec4 {
    let [r, g, b, a] = color.to_linear().to_f32_array();

    Vec4::new(r, g, b, a)
}


