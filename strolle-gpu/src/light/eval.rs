use core::f32::consts::PI;

use glam::Vec3;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{F32Ext, Hit};

pub fn distance_attenuation(
    distance_square: f32,
    inverse_range_squared: f32,
) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = (1.0 - factor * factor).saturate();
    let attenuation = smooth_factor * smooth_factor;

    attenuation / distance_square.max(0.0001)
}

pub fn diffuse(l: Vec3, v: Vec3, hit: Hit, roughness: f32, n_o_l: f32) -> f32 {
    let h = (l + v).normalize();
    let n_dot_v = hit.normal.dot(v).max(0.0001);
    let l_o_h = l.dot(h).saturate();

    fd_burley(roughness, n_dot_v, n_o_l, l_o_h)
}

pub fn specular(
    f0: Vec3,
    roughness: f32,
    n_o_v: f32,
    n_o_l: f32,
    n_o_h: f32,
    l_o_h: f32,
    specular_intensity: f32,
) -> Vec3 {
    let d = d_ggx(roughness, n_o_h);
    let v = v_smith_ggx_correlated(roughness, n_o_v, n_o_l);
    let f = fresnel(f0, l_o_h);

    specular_intensity * d * v * f
}

fn fd_burley(roughness: f32, n_o_v: f32, n_o_l: f32, l_o_h: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * l_o_h * l_o_h;
    let light_scatter = f_schlick(1.0, f90, n_o_l);
    let view_scatter = f_schlick(1.0, f90, n_o_v);

    light_scatter * view_scatter * (1.0 / PI)
}

fn d_ggx(roughness: f32, n_o_h: f32) -> f32 {
    let one_minus_noh_squared = 1.0 - n_o_h * n_o_h;
    let a = n_o_h * roughness;
    let k = roughness / (one_minus_noh_squared + a * a);

    k * k * (1.0 / PI)
}

fn v_smith_ggx_correlated(roughness: f32, n_o_v: f32, n_o_l: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambda_v = n_o_l * ((n_o_v - a2 * n_o_v) * n_o_v + a2).sqrt();
    let lambda_l = n_o_v * ((n_o_l - a2 * n_o_l) * n_o_l + a2).sqrt();

    0.5 / (lambda_v + lambda_l)
}

fn fresnel(f0: Vec3, l_o_h: f32) -> Vec3 {
    let f90 = f0.dot(Vec3::splat(50.0 * 0.33)).saturate();

    f_schlick_vec(f0, f90, l_o_h)
}

fn f_schlick(f0: f32, f90: f32, v_o_h: f32) -> f32 {
    f0 + (f90 - f0) * (1.0 - v_o_h).max(0.001).powf(5.0)
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_o_h: f32) -> Vec3 {
    f0 + (f90 - f0) * (1.0 - v_o_h).max(0.001).powf(5.0)
}
