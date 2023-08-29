use core::ops::{Deref, DerefMut};

use glam::{vec4, Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{LightId, Reservoir, Vec3Ext};

/// Reservoir for sampling direct lightning.
///
/// See: [`Reservoir`].
#[derive(Clone, Copy, Default)]
pub struct DirectReservoir {
    reservoir: Reservoir<DirectReservoirSample>,
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
        let d0 = unsafe { *buffer.get_unchecked(3 * id + 0) };
        let d1 = unsafe { *buffer.get_unchecked(3 * id + 1) };
        let d2 = unsafe { *buffer.get_unchecked(3 * id + 2) };

        Self {
            reservoir: Reservoir {
                sample: DirectReservoirSample {
                    light_id: LightId::new(d0.w.to_bits()),
                    light_radiance: d0.xyz(),
                    light_pdf: d1.w,
                    hit_point: d1.xyz(),
                },
                w_sum: Default::default(),
                m_sum: d2.x,
                w: d2.y,
            },
            frame: d2.z.to_bits(),
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = self
            .sample
            .light_radiance
            .extend(f32::from_bits(self.sample.light_id.get()));

        let d1 = self.sample.hit_point.extend(self.sample.light_pdf);

        let d2 = vec4(
            self.reservoir.m_sum,
            self.reservoir.w,
            f32::from_bits(self.frame),
            Default::default(),
        );

        unsafe {
            *buffer.get_unchecked_mut(3 * id + 0) = d0;
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

#[derive(Clone, Copy, Default)]
pub struct DirectReservoirSample {
    pub light_id: LightId,
    pub light_radiance: Vec3,
    pub light_pdf: f32,
    pub hit_point: Vec3,
}

impl DirectReservoirSample {
    pub fn p_hat(&self) -> f32 {
        self.light_radiance.luminance()
    }
}
