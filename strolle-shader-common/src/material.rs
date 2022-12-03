use crate::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Pod, Zeroable)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Material {
    // x,y,z is color, w is 1.0 indicates texture is present, 0.0 indicates texture is not present
    color: Vec4,
    // x,y,z is reflectivity color
    // w is to be reinterpreted as a u32:
    //   - lower 8 bits are the reflectivity parameter (0-255 mapped into 0.0-1.0)
    //   - next 8 bits are the emission parameter (0xff indicates an emissive material, 0x00 indicates a non-emissive material)
    reflectivity: Vec4,
}

impl Material {
    pub fn none() -> Self {
        Self {
            color: Default::default(),
            reflectivity: Default::default(),
        }
    }

    pub fn reflectivity_color(&self) -> Vec3 {
        self.reflectivity.xyz()
    }

    pub fn reflectivity(&self) -> f32 {
        let w = self.reflectivity.w.to_bits();

        (w & 0x000000ff) as f32 / 255.0
    }

    pub fn is_emissive(&self) -> bool {
        let w = self.reflectivity.w.to_bits();

        (w & 0x0000ff00) == 0x0000ff00
    }

    pub fn radiance(&self, world: &World, hit: Hit) -> Vec3 {
        let color = if self.has_texture() {
            (world.atlas_sample(hit.tri_id, hit) * self.color).truncate()
        } else {
            self.color.truncate()
        };

        if self.is_emissive() {
            return color;
        }

        let mut radiance = vec3(0.0, 0.0, 0.0);
        let mut light_idx = 0;

        while light_idx < world.lights.len() {
            let light = world.lights.get(light_idx);
            let ray = Ray::new(hit.point, light.pos() - hit.point);
            let distance = light.pos().distance(hit.point);

            let cone_factor = if light.is_spot() {
                let dir_light_to_hit = hit.point - light.pos();
                let dir_light_to_point = light.point_at() - light.pos();
                let angle = dir_light_to_point.angle_between(dir_light_to_hit);

                map_quadratic_clamped(angle, light.cone_angle())
            } else {
                1.0
            };

            if cone_factor > 0.0 {
                let diffuse_factor = if ray.hits_anything_up_to(world, distance)
                {
                    0.0
                } else {
                    ray.direction().dot(hit.normal).max(0.0)
                };

                radiance += cone_factor
                    * diffuse_factor
                    * light.color()
                    * light.intensity()
                    * color;
            }

            light_idx += 1;
        }

        radiance
    }

    pub fn has_texture(&self) -> bool {
        self.color.w == 1.0
    }
}

/// Remaps the given value with a quadratic formula.
/// Such that, the result is between 0.0 and 1.0
/// 1.0 is at v == 0.0
/// 0.0 is at abs(v) == span
fn map_quadratic_clamped(v: f32, span: f32) -> f32 {
    f32::clamp(1.0 - (v / span).powf(2.0), 0.0, 1.0)
}

#[cfg(not(target_arch = "spirv"))]
impl Material {
    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = color.extend(0.0);
        self
    }

    pub fn with_texture(mut self, val: bool) -> Self {
        self.color.w = if val { 1.0 } else { 0.0 };
        self
    }

    pub fn with_reflectivity(
        mut self,
        reflectivity: f32,
        reflection_color: Vec3,
    ) -> Self {
        let reflectivity = reflectivity.clamp(0.0, 1.0);
        let reflectivity = (reflectivity * 255.0) as u32;

        let mut w = self.reflectivity.w.to_bits();
        w ^= reflectivity & 0x000000ff;

        self.reflectivity = reflection_color.extend(f32::from_bits(w));
        self
    }

    pub fn with_emissive(mut self, emissive: bool) -> Self {
        let mut w = self.reflectivity.w.to_bits();
        let v = if emissive { 0x0000ff00 } else { 0x00000000 };

        w |= v;

        self.reflectivity.w = f32::from_bits(w);
        self
    }
}

#[cfg(not(target_arch = "spirv"))]
impl Default for Material {
    fn default() -> Self {
        Material {
            color: vec4(1.0, 1.0, 1.0, 0.0),
            reflectivity: vec4(0.0, 0.0, 0.0, 0.0),
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
