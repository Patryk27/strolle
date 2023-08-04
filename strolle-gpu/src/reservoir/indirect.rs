use core::f32::consts::PI;
use core::ops::{Deref, DerefMut};

use glam::{vec3, UVec2, Vec3, Vec4, Vec4Swizzles};

use crate::{F32Ext, Hit, Reservoir, SpecularBrdf};

#[derive(Clone, Copy, Default)]
pub struct IndirectReservoir {
    reservoir: Reservoir<IndirectReservoirSample>,
}

impl IndirectReservoir {
    pub fn new(sample: IndirectReservoirSample, p_hat: f32) -> Self {
        Self {
            reservoir: Reservoir::new(sample, p_hat),
        }
    }

    pub fn expects_diffuse_sample(screen_pos: UVec2, frame: u32) -> bool {
        (screen_pos.x % 2) == (frame % 2)
    }

    pub fn expects_specular_sample(screen_pos: UVec2, frame: u32) -> bool {
        !Self::expects_diffuse_sample(screen_pos, frame)
    }

    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.get_unchecked(4 * id + 0) };
        let d1 = unsafe { *buffer.get_unchecked(4 * id + 1) };
        let d2 = unsafe { *buffer.get_unchecked(4 * id + 2) };
        let d3 = unsafe { *buffer.get_unchecked(4 * id + 3) };

        Self {
            reservoir: Reservoir {
                sample: IndirectReservoirSample {
                    radiance: d0.xyz(),
                    hit_point: d1.xyz(),
                    sample_point: d2.xyz(),
                    sample_normal: d3.xyz(),
                    frame: d2.w.to_bits(),
                },
                w_sum: Default::default(),
                m_sum: d0.w,
                w: d1.w,
            },
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = self.sample.radiance.extend(self.m_sum);
        let d1 = self.sample.hit_point.extend(self.w);

        let d2 = self
            .sample
            .sample_point
            .extend(f32::from_bits(self.sample.frame));

        let d3 = self.sample.sample_normal.extend(Default::default());

        unsafe {
            *buffer.get_unchecked_mut(4 * id + 0) = d0;
            *buffer.get_unchecked_mut(4 * id + 1) = d1;
            *buffer.get_unchecked_mut(4 * id + 2) = d2;
            *buffer.get_unchecked_mut(4 * id + 3) = d3;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sample.frame == 0
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
    pub frame: u32,
}

impl IndirectReservoirSample {
    pub fn temporal_p_hat(&self) -> f32 {
        self.radiance.dot(vec3(0.2126, 0.7152, 0.0722))
    }

    pub fn spatial_p_hat(&self, point: Vec3, normal: Vec3) -> f32 {
        self.temporal_p_hat() * self.direction(point).dot(normal).max(0.0)
    }

    pub fn direction(&self, point: Vec3) -> Vec3 {
        (self.sample_point - point).normalize()
    }

    pub fn cosine(&self, direct_hit: &Hit) -> f32 {
        direct_hit
            .gbuffer
            .normal
            .dot(self.direction(direct_hit.point))
            .max(0.0)
    }

    pub fn diffuse_brdf(&self) -> f32 {
        1.0 / PI
    }

    pub fn specular_brdf(&self, direct_hit: &Hit) -> f32 {
        let l = (self.sample_point - direct_hit.point).normalize();
        let v = (direct_hit.origin - direct_hit.point).normalize();
        let n = direct_hit.gbuffer.normal;

        SpecularBrdf::new(&direct_hit.gbuffer)
            .eval_f(l, v, n)
            .clamp(0.0, 1.0)
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
        let cosine = self.sample_normal.dot(vec / distance).saturate();

        (distance, cosine)
    }
}
