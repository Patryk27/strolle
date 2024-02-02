use core::ops::{Deref, DerefMut};

use glam::{vec4, Vec3, Vec4, Vec4Swizzles};
use spirv_std::arch::IndexUnchecked;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::utils::U32Ext;
use crate::{Hit, LightId, LightsView, Ray, Reservoir, Vec3Ext};

#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DiReservoir {
    pub reservoir: Reservoir<DiSample>,
}

impl DiReservoir {
    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.index_unchecked(2 * id) };
        let d1 = unsafe { *buffer.index_unchecked(2 * id + 1) };

        Self {
            reservoir: Reservoir {
                sample: DiSample {
                    light_id: LightId::new(d1.w.to_bits()),
                    light_point: d1.xyz(),
                    exists: d0.w.to_bits() > 0,
                },
                m: d0.x,
                w: d0.y,
            },
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = vec4(
            self.reservoir.m,
            self.reservoir.w,
            0.0,
            f32::from_bits(u32::from_bytes([
                self.sample.exists as u32,
                0,
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

    pub fn is_empty(&self) -> bool {
        !self.sample.exists
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
    pub light_id: LightId,
    pub light_point: Vec3,
    pub exists: bool,
}

impl DiSample {
    pub fn is_valid(&self, lights: LightsView) -> bool {
        if self.light_id.get() >= lights.len() as u32 {
            return false;
        }

        let light = lights.get(self.light_id);

        light.center().distance(self.light_point) <= light.radius()
    }

    pub fn pdf(&self, lights: LightsView, hit: Hit) -> f32 {
        lights.get(self.light_id).radiance(hit).perc_luma()
    }

    pub fn ray(&self, hit: Hit) -> Ray {
        let dir = hit.point - self.light_point;

        Ray::new(self.light_point, dir.normalize()).with_length(dir.length())
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
                        light_id: LightId::new(3 * idx as u32),
                        light_point: vec3(1.0, 2.0, 3.0 + (idx as f32)),
                        exists: idx as u32 % 2 == 0,
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
