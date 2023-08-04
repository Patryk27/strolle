use glam::{vec4, Vec2, Vec3, Vec4, Vec4Swizzles};

use crate::{Normal, U32Ext};

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
        let base_color = {
            let [x, y, z, w] = d0.x.to_bits().to_bytes();

            vec4(
                x as f32 / 255.0,
                y as f32 / 255.0,
                z as f32 / 255.0,
                w as f32 / 255.0,
            )
        };

        let normal = Normal::decode(d0.yz());

        let (metallic, roughness) = {
            let [metallic, roughness, ..] = d0.w.to_bits().to_bytes();

            let metallic = metallic as f32 / 255.0;
            let roughness = roughness as f32 / 255.0;

            (metallic, roughness)
        };

        let emissive = d1.xyz();
        let depth = d1.w;

        Self {
            base_color,
            normal,
            metallic,
            emissive,
            roughness,
            reflectance: 0.5, // TODO
            depth,
        }
    }

    pub fn pack(self) -> [Vec4; 2] {
        let d0 = {
            let x = {
                let base_color =
                    self.base_color.clamp(Vec4::ZERO, Vec4::ONE) * 255.0;

                let base_color = base_color.as_uvec4();

                f32::from_bits(u32::from_bytes([
                    base_color.x as u32,
                    base_color.y as u32,
                    base_color.z as u32,
                    base_color.w as u32,
                ]))
            };

            let Vec2 { x: y, y: z } = Normal::encode(self.normal);

            let w = {
                let metallic = self.metallic.clamp(0.0, 1.0) * 255.0;
                let roughness = self.roughness.clamp(0.0, 1.0) * 255.0;

                f32::from_bits(u32::from_bytes([
                    metallic as u32,
                    roughness as u32,
                    Default::default(),
                    Default::default(),
                ]))
            };

            vec4(x, y, z, w)
        };

        let d1 = self.emissive.extend(self.depth);

        [d0, d1]
    }

    pub fn is_some(&self) -> bool {
        self.depth != Default::default()
    }

    pub fn clamped_roughness(&self) -> f32 {
        self.roughness.clamp(0.089 * 0.089, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use glam::vec3;

    use super::*;

    const EPSILON: f32 = 0.01;

    #[test]
    fn serialization() {
        let target = GBufferEntry {
            base_color: vec4(0.1, 0.2, 0.3, 0.4),
            normal: vec3(0.26, 0.53, 0.80),
            metallic: 0.33,
            emissive: vec3(2.0, 3.0, 4.0),
            roughness: 0.66,
            reflectance: 0.5,
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

        assert_relative_eq!(target.roughness, 0.66, epsilon = EPSILON);
        assert_relative_eq!(target.reflectance, 0.5, epsilon = EPSILON);
        assert_relative_eq!(target.depth, 123.456, epsilon = EPSILON);
    }
}
