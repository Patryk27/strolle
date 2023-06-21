use core::ops::{Deref, DerefMut};

use glam::{vec3, vec4, Vec3, Vec4, Vec4Swizzles};

use crate::Noise;

#[derive(Clone, Copy, Default)]
pub struct Reservoir<T> {
    pub sample: T,
    pub w_sum: f32,
    pub m_sum: f32,
    pub w: f32,
}

impl<T> Reservoir<T>
where
    T: Clone + Copy,
{
    pub fn new(sample: T, weight: f32) -> Self {
        Self {
            sample,
            w_sum: weight,
            w: 1.0,
            m_sum: if weight == 0.0 { 0.0 } else { 1.0 },
        }
    }

    pub fn add(&mut self, noise: &mut Noise, s_new: T, w_new: f32) -> bool {
        self.w_sum += w_new;
        self.m_sum += 1.0;

        if noise.sample() <= w_new / self.w_sum {
            self.sample = s_new;
            true
        } else {
            false
        }
    }

    pub fn merge(&mut self, noise: &mut Noise, r: &Self, p_hat: f32) -> bool {
        if r.m_sum <= 0.0 {
            return false;
        }

        self.m_sum += r.m_sum - 1.0;
        self.add(noise, r.sample, r.w * r.m_sum * p_hat)
    }

    pub fn normalize(&mut self, p_hat: f32, max_w: f32, max_m_sum: f32) {
        self.w = self.w_sum / (self.m_sum * p_hat).max(0.001);
        self.w = self.w.min(max_w);
        self.m_sum = self.m_sum.min(max_m_sum);
    }
}

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
        let d0 = buffer[2 * id];
        let d1 = buffer[2 * id + 1];

        Self {
            reservoir: Reservoir {
                sample: DirectReservoirSample {
                    light_id: d0.w.to_bits(),
                    light_contribution: d0.xyz(),
                },
                w_sum: Default::default(),
                m_sum: d1.x,
                w: d1.y,
            },
            frame: d1.z.to_bits(),
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = self
            .sample
            .light_contribution
            .extend(f32::from_bits(self.sample.light_id));

        let d1 = vec4(
            self.reservoir.m_sum,
            self.reservoir.w,
            f32::from_bits(self.frame),
            Default::default(),
        );

        buffer[2 * id] = d0;
        buffer[2 * id + 1] = d1;
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

#[derive(Clone, Copy)]
pub struct DirectReservoirSample {
    pub light_id: u32, // TODO use LightId
    pub light_contribution: Vec3,
}

impl DirectReservoirSample {
    pub fn p_hat(&self) -> f32 {
        self.light_contribution.dot(vec3(0.2126, 0.7152, 0.0722))
    }
}

impl Default for DirectReservoirSample {
    fn default() -> Self {
        Self {
            light_id: u32::MAX,
            light_contribution: Default::default(),
        }
    }
}

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
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
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
        let distance = vec.length().max(0.001);
        let cosine = self.sample_normal.dot(vec / distance).clamp(0.0, 1.0);

        (distance, cosine)
    }
}
