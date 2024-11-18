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

        let packed = d0.w.to_bits();
        let metallic_u = (packed >> 24) & 0xFF;
        let roughness_u = (packed >> 16) & 0xFF;
        let reflectance_u = (packed >> 8) & 0xFF;
        let emissive_r_u = packed & 0xFF;

        let metallic = metallic_u as f32 / 255.0;
        let roughness = roughness_u as f32 / 255.0;
        let reflectance = reflectance_u as f32 / 255.0;
        let emissive_r = emissive_r_u as f32 / 255.0;

        let w_scaled = d1.w as u32;
        let emissive_g_u = (w_scaled >> 8) & 0xFF;
        let emissive_b_u = w_scaled & 0xFF;

        let emissive_g = emissive_g_u as f32 / 255.0;
        let emissive_b = emissive_b_u as f32 / 255.0;

        let emissive = Vec3::new(emissive_r, emissive_g, emissive_b);

        let base_color = vec4(d1.x, d1.y, d1.z, 1.0); // Assuming alpha is 1.0

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
        // d0: Rgba32Float
        let d0 = {
            let x = self.depth; // f32
            let Vec2 { x: y, y: z } = Normal::encode(self.normal); // Encoded normal components

            // Pack metallic, roughness, reflectance, and emissive_r into a u32
            let metallic_u = (self.metallic.clamp(0.0, 1.0) * 255.0).round() as u32;
            let roughness_u = (self.roughness.clamp(0.0, 1.0) * 255.0).round() as u32;
            let reflectance_u = (self.reflectance.clamp(0.0, 1.0) * 255.0).round() as u32;
            let emissive_r_u = (self.emissive.x.clamp(0.0, 1.0) * 255.0).round() as u32;

            let packed = (metallic_u << 24)
                | (roughness_u << 16)
                | (reflectance_u << 8)
                | emissive_r_u;

            let w = f32::from_bits(packed);

            vec4(x, y, z, w)
        };

        // d1: Rgba16Float
        let d1 = {
            let base_color = self.base_color.clamp(Vec4::ZERO, Vec4::ONE);

            let emissive_g_u = (self.emissive.y.clamp(0.0, 1.0) * 255.0).round() as u32;
            let emissive_b_u = (self.emissive.z.clamp(0.0, 1.0) * 255.0).round() as u32;

            let emissive_packed = (emissive_g_u << 8) | emissive_b_u;
            let w = emissive_packed as f32;

            vec4(
                base_color.x,
                base_color.y,
                base_color.z,
                w, // Store packed emissive_g and emissive_b
            )
        };

        [d0, d1]
    }

    pub fn is_some(self) -> bool {
        self.depth != Default::default()
    }

    pub fn clamped_roughness(self) -> f32 {
        self.roughness.clamp(0.089 * 0.089, 1.0)
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
        assert_relative_eq!(target.base_color.w, 0.4, epsilon = 0.1);

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
