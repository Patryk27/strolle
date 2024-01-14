use glam::{vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{F32Ext, Normal, U32Ext};

#[derive(Clone, Copy, Default)]
pub struct GBufferEntry {
    pub base_color: Vec4,
    pub normal: Vec3,
    pub metallic: f32,
    pub emissive: Vec3,
    pub roughness: f32,
    pub reflectance: f32,
    pub depth: f32,
}

impl GBufferEntry {
    pub fn unpack([d0, d1]: [Vec4; 2]) -> Self {
        let depth = d0.x;
        let normal = Normal::decode(d0.yz());

        let (metallic, roughness, reflectance) = {
            let [metallic, roughness, reflectance, ..] =
                d0.w.to_bits().to_bytes();

            let metallic = metallic as f32 / 255.0;
            let roughness = (roughness as f32 / 255.0).sqr();
            let reflectance = reflectance as f32 / 255.0;

            (metallic, roughness, reflectance)
        };

        let emissive = d1.xyz();

        let base_color = {
            let [x, y, z, w] = d1.w.to_bits().to_bytes();

            vec4(
                x as f32 / 255.0,
                y as f32 / 255.0,
                z as f32 / 255.0,
                w as f32 / 255.0,
            )
            .powf(2.2)
        };

        Self {
            base_color,
            normal,
            metallic,
            emissive,
            roughness,
            reflectance,
            depth,
        }
    }

    pub fn pack(self) -> [Vec4; 2] {
        let d0 = {
            let x = self.depth;
            let Vec2 { x: y, y: z } = Normal::encode(self.normal);

            let w = {
                let metallic = self.metallic.clamp(0.0, 1.0) * 255.0;
                let roughness = self.roughness.sqrt().clamp(0.0, 1.0) * 255.0;
                let reflectance = self.reflectance.clamp(0.0, 1.0) * 255.0;

                f32::from_bits(u32::from_bytes([
                    metallic as u32,
                    roughness as u32,
                    reflectance as u32,
                    Default::default(),
                ]))
            };

            vec4(x, y, z, w)
        };

        let d1 = {
            // TODO doesn't need to use as much space
            let x = self.emissive.x;
            let y = self.emissive.y;
            let z = self.emissive.z;

            let w = {
                let base_color = self
                    .base_color
                    .powf(1.0 / 2.2)
                    .clamp(Vec4::ZERO, Vec4::ONE);

                let base_color = (base_color * 255.0).as_uvec4();

                f32::from_bits(u32::from_bytes([
                    base_color.x,
                    base_color.y,
                    base_color.z,
                    base_color.w,
                ]))
            };

            vec4(x, y, z, w)
        };

        [d0, d1]
    }

    pub fn is_some(&self) -> bool {
        self.depth != Default::default()
    }

    pub fn clamped_roughness(&self) -> f32 {
        self.roughness.clamp(0.089 * 0.089, 1.0)
    }

    pub fn is_mirror(&self) -> bool {
        self.roughness == 0.0
    }

    pub fn needs_diff(&self) -> bool {
        self.metallic < 1.0
    }

    pub fn needs_spec(&self) -> bool {
        self.metallic > 0.0
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::vec3;

    use super::*;

    const EPSILON: f32 = 0.005;

    #[test]
    fn serialization() {
        let target = GBufferEntry {
            base_color: vec4(0.1, 0.2, 0.3, 0.4),
            normal: vec3(0.26, 0.53, 0.80),
            metallic: 0.33,
            emissive: vec3(2.0, 3.0, 4.0),
            roughness: 0.05,
            reflectance: 0.25,
            depth: 123.456,
        };

        let target = GBufferEntry::unpack(target.pack());

        assert_relative_eq!(target.base_color.x, 0.1, epsilon = EPSILON);
        assert_relative_eq!(target.base_color.y, 0.2, epsilon = EPSILON);
        assert_relative_eq!(target.base_color.z, 0.3, epsilon = EPSILON);
        assert_relative_eq!(target.base_color.w, 0.4, epsilon = EPSILON);

        assert_relative_eq!(target.normal.x, 0.26, epsilon = EPSILON);
        assert_relative_eq!(target.normal.y, 0.53, epsilon = EPSILON);
        assert_relative_eq!(target.normal.z, 0.80, epsilon = EPSILON);

        assert_relative_eq!(target.metallic, 0.33, epsilon = EPSILON);

        assert_relative_eq!(target.emissive.x, 2.0, epsilon = EPSILON);
        assert_relative_eq!(target.emissive.y, 3.0, epsilon = EPSILON);
        assert_relative_eq!(target.emissive.z, 4.0, epsilon = EPSILON);

        assert_relative_eq!(target.roughness, 0.05, epsilon = EPSILON);
        assert_relative_eq!(target.reflectance, 0.25, epsilon = EPSILON);
        assert_relative_eq!(target.depth, 123.456, epsilon = EPSILON);
    }
}
