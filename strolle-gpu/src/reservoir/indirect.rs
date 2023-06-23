use core::ops::{Deref, DerefMut};

use glam::{vec3, Vec3, Vec4, Vec4Swizzles};

use crate::Reservoir;

#[derive(Clone, Copy, Default)]
pub struct IndirectReservoir {
    reservoir: Reservoir<IndirectReservoirSample>,
    pub frame: u32,
}

impl IndirectReservoir {
    pub fn new(
        sample: IndirectReservoirSample,
        p_hat: f32,
        frame: u32,
    ) -> Self {
        Self {
            reservoir: Reservoir::new(sample, p_hat),
            frame,
        }
    }

    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = buffer[4 * id];
        let d1 = buffer[4 * id + 1];
        let d2 = buffer[4 * id + 2];
        let d3 = buffer[4 * id + 3];

        Self {
            reservoir: Reservoir {
                sample: IndirectReservoirSample {
                    radiance: d0.xyz(),
                    hit_point: d1.xyz(),
                    sample_point: d2.xyz(),
                    sample_normal: d3.xyz(),
                },
                w_sum: Default::default(),
                m_sum: d0.w,
                w: d1.w,
            },
            frame: d2.w.to_bits(),
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = self.sample.radiance.extend(self.m_sum);
        let d1 = self.sample.hit_point.extend(self.w);
        let d2 = self.sample.sample_point.extend(f32::from_bits(self.frame));
        let d3 = self.sample.sample_normal.extend(Default::default());

        buffer[4 * id] = d0;
        buffer[4 * id + 1] = d1;
        buffer[4 * id + 2] = d2;
        buffer[4 * id + 3] = d3;
    }

    pub fn age(&self, frame: u32) -> u32 {
        frame - self.frame
    }
}

impl Deref for IndirectReservoir {
    type Target = Reservoir<IndirectReservoirSample>;

    fn deref(&self) -> &Self::Target {
        &self.reservoir
    }
}

impl DerefMut for IndirectReservoir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reservoir
    }
}

#[derive(Clone, Copy, Default)]
pub struct IndirectReservoirSample {
    pub radiance: Vec3,
    pub hit_point: Vec3,
    pub sample_point: Vec3,
    pub sample_normal: Vec3,
}

impl IndirectReservoirSample {
    pub fn p_hat(&self) -> f32 {
        self.radiance.dot(vec3(0.2126, 0.7152, 0.0722))
    }

    pub fn jacobian(&self, new_hit_point: Vec3) -> f32 {
        let (new_distance, new_cosine) = self.partial_jacobian(new_hit_point);

        let (orig_distance, orig_cosine) =
            self.partial_jacobian(self.hit_point);

        let x = new_cosine * orig_distance * orig_distance;
        let y = orig_cosine * new_distance * new_distance;

        if y == 0.0 {
            0.0
        } else {
            x / y
        }
    }

    fn partial_jacobian(&self, hit_point: Vec3) -> (f32, f32) {
        let vec = hit_point - self.sample_point;
        let distance = vec.length();
        let cosine = self.sample_normal.dot(vec / distance).clamp(0.0, 1.0);

        (distance, cosine)
    }
}
