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

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DiReservoirData {
    pub m: f32,
    pub w: f32,
    pub pdf: f32,
    pub confidence: f32,
    pub is_occluded: u32,
    pub light_id: u32,
    // Add padding to align `light_point` to 16 bytes
    _padding0: [u32; 2],
    pub light_point: Vec3
}

impl DiReservoir {
    pub fn read(buffer: &[DiReservoirData], id: usize) -> Self {
        let data = unsafe { *buffer.index_unchecked(id) };
        Self {
            reservoir: Reservoir {
                sample: DiSample {
                    pdf: data.pdf,
                    confidence: data.confidence,
                    light_id: LightId::new(data.light_id),
                    light_point: data.light_point,
                    is_occluded: data.is_occluded != 0,
                },
                m: data.m,
                w: data.w,
            },
        }
    }

    pub fn write(self, buffer: &mut [DiReservoirData], id: usize) {
        let data = DiReservoirData {
            m: self.reservoir.m,
            w: self.reservoir.w,
            pdf: self.sample.pdf,
            confidence: self.sample.confidence,
            is_occluded: self.sample.is_occluded as u32,
            light_id: self.sample.light_id.get(),
            _padding0: [0u32; 2],
            light_point: self.sample.light_point
        };
        unsafe {
            *buffer.index_unchecked_mut(id) = data;
        }
    }

    pub fn copy(input: &[DiReservoirData], output: &mut [DiReservoirData], id: usize) {
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

        Ray::new(self.light_point, crate::safe_normalize(dir)).with_len(dir.length())
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

        let mut buffer = [DiReservoirData::default(), DiReservoirData::default()];

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
