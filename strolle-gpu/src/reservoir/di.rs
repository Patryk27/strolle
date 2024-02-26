use core::ops::{Deref, DerefMut};

use glam::{vec4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::arch::IndexUnchecked;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Hit, Light, LightId, LightsView, Ray, Reservoir, U32Ext, Vec3Ext};

#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DiReservoir {
    pub reservoir: Reservoir<DiSample>,
}

impl DiReservoir {
    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.index_unchecked(2 * id) };
        let d1 = unsafe { *buffer.index_unchecked(2 * id + 1) };
        let [is_occluded, confidence, ..] = d0.w.to_bits().to_bytes();

        Self {
            reservoir: Reservoir {
                sample: DiSample {
                    pdf: d0.z,
                    confidence: confidence as f32,
                    light_id: LightId::new(d1.w.to_bits()),
                    light_point: d1.xyz(),
                    is_occluded: is_occluded > 0,
                },
                m: d0.x,
                w: d0.y,
            },
        }
    }

    pub fn write(self, buffer: &mut [Vec4], id: usize) {
        let d0 = vec4(
            self.reservoir.m,
            self.reservoir.w,
            self.sample.pdf,
            f32::from_bits(u32::from_bytes([
                self.sample.is_occluded as u32,
                self.sample.confidence as u32,
                0,
                0,
            ])),
        );

        let d1 = self
            .sample
            .light_point
            .extend(f32::from_bits(self.sample.light_id.get()));

        unsafe {
            *buffer.index_unchecked_mut(2 * id) = d0;
            *buffer.index_unchecked_mut(2 * id + 1) = d1;
        }
    }

    pub fn copy(input: &[Vec4], output: &mut [Vec4], id: usize) {
        // TODO optimize
        Self::read(input, id).write(output, id);
    }

    pub fn is_empty(self) -> bool {
        self.m == 0.0
    }
}

impl Deref for DiReservoir {
    type Target = Reservoir<DiSample>;

    fn deref(&self) -> &Self::Target {
        &self.reservoir
    }
}

impl DerefMut for DiReservoir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reservoir
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DiSample {
    pub pdf: f32,
    pub confidence: f32, // TODO consider storing inside the reservoir instead
    pub light_id: LightId,
    pub light_point: Vec3,
    pub is_occluded: bool,
}

impl DiSample {
    pub fn pdf(self, lights: LightsView, hit: Hit) -> f32 {
        let light = lights.get(self.light_id);

        self.pdf_ex(light, hit)
    }

    pub fn pdf_prev(self, lights: LightsView, hit: Hit) -> f32 {
        let light = lights.get_prev(self.light_id);

        self.pdf_ex(light, hit)
    }

    fn pdf_ex(self, light: Light, mut hit: Hit) -> f32 {
        hit.gbuffer.base_color = Vec4::ONE;

        if !light.is_none() && light.contains(self.light_point) {
            // TODO use a cheaper proxy
            light.radiance(hit).sum().luma()
        } else {
            0.0
        }
    }

    pub fn ray(self, hit_point: Vec3) -> Ray {
        let dir = hit_point - self.light_point;

        Ray::new(self.light_point, dir.normalize()).with_len(dir.length())
    }
}

#[cfg(test)]
mod tests {
    use glam::vec3;

    use super::*;

    #[test]
    fn serialization() {
        fn target(idx: usize) -> DiReservoir {
            DiReservoir {
                reservoir: Reservoir {
                    sample: DiSample {
                        pdf: 123.0,
                        confidence: (idx % 2 == 0) as u32 as f32,
                        light_id: LightId::new(3 * idx as u32),
                        light_point: vec3(1.0, 2.0, 3.0 + (idx as f32)),
                        is_occluded: idx as u32 % 2 == 0,
                    },
                    m: 11.0,
                    w: 12.0 + (idx as f32),
                },
            }
        }

        let mut buffer = [Vec4::ZERO; 2 * 10];

        for idx in 0..10 {
            target(idx).write(&mut buffer, idx);
        }

        for idx in 0..10 {
            let actual = DiReservoir::read(&buffer, idx);
            let expected = target(idx);

            assert_eq!(expected, actual);
        }
    }
}
