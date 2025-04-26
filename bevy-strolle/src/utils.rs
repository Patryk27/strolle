use bevy::prelude::*;

pub fn color_to_vec3(color: Color) -> Vec3 {
    color.to_linear().to_vec3()
}

pub fn color_to_vec4(color: Color) -> Vec4 {
    color.to_linear().to_vec4()
}
