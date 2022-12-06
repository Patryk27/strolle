use core::f32::consts::PI;

use crate::*;

/// # Memory model
///
/// ```
/// base_color.x = base color's red component
/// base_color.y = base color's green component
/// base_color.z = base color's blue component
/// base_color.w = base color's alpha component
/// ```
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    base_color: Vec4,
}

impl Material {
    pub fn none() -> Self {
        Self {
            base_color: Default::default(),
        }
    }

    // TODO
    pub fn reflectivity(&self) -> f32 {
        0.0
    }

    // TODO
    pub fn reflectivity_color(&self) -> Vec3 {
        vec3(0.0, 0.0, 0.0)
    }

    pub fn radiance(&self, world: &World, hit: Hit) -> Vec4 {
        let mut radiance = vec4(0.0, 0.0, 0.0, self.base_color.w);

        let diffuse_color = self.base_color;
        let mut light_idx = 0;

        while light_idx < world.lights.len() {
            // TODO should be configurable (per light)
            let perceptual_roughness = 0.089;

            // TODO should be configurable (per light)
            let range = 20.0f32;

            let light = world.lights.get(light_idx);
            let vec = light.pos() - hit.point;
            let distance_squared = vec.length_squared();
            let distance = distance_squared.sqrt(); // TODO: sqrt is expensive

            let ray = Ray::new(hit.point, vec);
            let l = vec.normalize();
            let h = (l + ray.direction()).normalize();
            let n_dot_v = hit.normal.dot(ray.direction()).max(0.0001);
            let n_o_l = hit.normal.dot(l).clamp(0.0, 1.0);
            let l_o_h = l.dot(h).clamp(0.0, 1.0);
            let roughness = perceptual_roughness * perceptual_roughness;

            let diffuse_factor = if ray.hits_anything_up_to(world, distance) {
                0.0
            } else {
                fd_burley(roughness, n_dot_v, n_o_l, l_o_h)
            };

            let distance_attenuation =
                distance_attenuation(distance_squared, 1.0 / range.powf(2.0));

            let contribution = light.color()
                * diffuse_color.xyz()
                * diffuse_factor
                * distance_attenuation
                * n_o_l;

            radiance += contribution.extend(0.0);
            light_idx += 1;
        }

        radiance
    }
}

// Thanks to https://google.github.io/filament/Filament.html
fn fd_burley(roughness: f32, n_o_v: f32, n_o_l: f32, l_o_h: f32) -> f32 {
    fn f_schlick(f0: f32, f90: f32, v_o_h: f32) -> f32 {
        f0 + (f90 - f0) * (1.0 - v_o_h).powf(5.0)
    }

    let f90 = 0.5 + 2.0 * roughness * l_o_h * l_o_h;
    let light_scatter = f_schlick(1.0, f90, n_o_l);
    let view_scatter = f_schlick(1.0, f90, n_o_v);

    light_scatter * view_scatter * (1.0 / PI)
}

// Thanks to Bevy's `pbr_lightning.wgsl`
fn distance_attenuation(
    distance_square: f32,
    inverse_range_squared: f32,
) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = (1.0 - factor * factor).clamp(0.0, 1.0);
    let attenuation = smooth_factor * smooth_factor;

    attenuation * 1.0 / distance_square.max(0.0001)
}

#[cfg(not(target_arch = "spirv"))]
impl Material {
    pub fn with_base_color(mut self, base_color: Vec4) -> Self {
        self.base_color = base_color;
        self
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct MaterialId(usize);

impl MaterialId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn get(self) -> usize {
        self.0
    }
}
