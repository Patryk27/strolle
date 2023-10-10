use core::ops::{Deref, DerefMut};

use glam::{vec4, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{Hit, LightId, LightsView, Ray, Reservoir, Vec3Ext};

/// Reservoir for sampling direct lightning.
///
/// See: [`Reservoir`].
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectReservoir {
    pub reservoir: Reservoir<DirectReservoirSample>,
    pub frame: u32,
}

impl DirectReservoir {
    pub fn new(sample: DirectReservoirSample, p_hat: f32, frame: u32) -> Self {
        Self {
            reservoir: Reservoir::new(sample, p_hat),
            frame,
        }
    }

    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.get_unchecked(3 * id) };
        let d1 = unsafe { *buffer.get_unchecked(3 * id + 1) };
        let d2 = unsafe { *buffer.get_unchecked(3 * id + 2) };

        Self {
            reservoir: Reservoir {
                sample: DirectReservoirSample {
                    light_id: LightId::new(d1.w.to_bits()),
                    light_position: d1.xyz(),
                    surface_point: d2.xyz(),
                },
                w_sum: Default::default(),
                m_sum: d0.x,
                w: d0.y,
            },
            frame: d0.z.to_bits(),
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = vec4(
            self.reservoir.m_sum,
            self.reservoir.w,
            f32::from_bits(self.frame),
            Default::default(),
        );

        let d1 = self
            .sample
            .light_position
            .extend(f32::from_bits(self.sample.light_id.get()));

        let d2 = self.sample.surface_point.extend(0.0);

        unsafe {
            *buffer.get_unchecked_mut(3 * id) = d0;
            *buffer.get_unchecked_mut(3 * id + 1) = d1;
            *buffer.get_unchecked_mut(3 * id + 2) = d2;
        }
    }

    pub fn age(&self, frame: u32) -> u32 {
        frame - self.frame
    }
}

impl Deref for DirectReservoir {
    type Target = Reservoir<DirectReservoirSample>;

    fn deref(&self) -> &Self::Target {
        &self.reservoir
    }
}

impl DerefMut for DirectReservoir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reservoir
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct DirectReservoirSample {
    pub light_id: LightId,
    pub light_position: Vec3,
    pub surface_point: Vec3,
}

impl DirectReservoirSample {
    pub fn p_hat(&self, lights: LightsView, hit: Hit) -> f32 {
        lights.get(self.light_id).radiance(hit).luminance()
    }

    pub fn ray(&self, hit: Hit) -> (Ray, f32) {
        let dir = hit.point - self.light_position;
        let ray = Ray::new(self.light_position, dir.normalize());

        (ray, dir.length())
    }
}

#[cfg(test)]
mod tests {
    use glam::vec3;

    use super::*;

    #[test]
    fn serialization() {
        fn target(idx: usize) -> DirectReservoir {
            DirectReservoir {
                reservoir: Reservoir {
                    sample: DirectReservoirSample {
                        light_id: LightId::new(1234),
                        light_position: vec3(1.0, 2.0, 3.0),
                        surface_point: vec3(4.0, 5.0, 6.0),
                    },
                    w_sum: 10.0,
                    m_sum: 11.0,
                    w: 12.0,
                },
                frame: 100 + (idx as u32),
            }
        }

        let mut buffer = [Vec4::ZERO; 3 * 10];

        for idx in 0..10 {
            target(idx).write(&mut buffer, idx);
        }

        for idx in 0..10 {
            let actual = DirectReservoir::read(&buffer, idx);

            let expected = {
                let mut t = target(idx);

                t.w_sum = 0.0;
                t
            };

            assert_eq!(expected, actual);
        }
    }
}
