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
        let color = self.base_color;
        let mut radiance = color;
        let mut light_idx = 0;

        while light_idx < world.lights.len() {
            let light = world.lights.get(light_idx);
            let ray = Ray::new(hit.point, light.pos() - hit.point);
            let dir_light_to_hit = hit.point - light.pos();

            let distance_squared = dir_light_to_hit.dot(dir_light_to_hit);
            let distance = distance_squared.sqrt(); // TODO: sqrt is expensive

            let cone_factor = if light.is_spot() {
                let dir_light_to_point = light.point_at() - light.pos();
                let angle = dir_light_to_point.angle_between(dir_light_to_hit);

                map_quadratic_clamped(angle, light.cone_angle())
            } else {
                1.0
            };

            let diffuse_factor = if cone_factor > 0.0 {
                if ray.hits_anything_up_to(world, distance) {
                    0.0
                } else {
                    ray.direction().dot(hit.normal).max(0.0)
                }
            } else {
                0.0
            };

            // TODO: Add range (or range squared?) as light param
            //       for now this is the default bevy value for point lights
            const LIGHT_RANGE: f32 = 20.0;

            let distance_attenuation = distance_attenuation(
                distance_squared,
                1.0 / LIGHT_RANGE.powf(2.0),
            );

            let light_radiance = cone_factor
                * diffuse_factor
                * light.color()
                * light.intensity()
                * distance_attenuation
                * color.xyz();

            radiance += light_radiance.extend(0.0);
            light_idx += 1;
        }

        radiance
    }
}

/// Remaps given value with a quadratic formula so that the result is between
/// 0.0 and 1.0
///
/// 1.0 is at v == 0.0
/// 0.0 is at abs(v) == span
fn map_quadratic_clamped(v: f32, span: f32) -> f32 {
    f32::clamp(1.0 - (v / span).powf(2.0), 0.0, 1.0)
}

fn distance_attenuation(
    distance_square: f32,
    inverse_range_squared: f32,
) -> f32 {
    let factor = distance_square * inverse_range_squared;
    let smooth_factor = (1.0 - factor * factor).clamp(0.0, 1.0);
    let attenuation = smooth_factor * smooth_factor;

    attenuation * 1.0 / f32::max(distance_square, 0.0001)
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
