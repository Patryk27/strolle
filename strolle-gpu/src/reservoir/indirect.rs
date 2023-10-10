use core::f32::consts::PI;
use core::ops::{Deref, DerefMut};

use glam::{Vec3, Vec4, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{BrdfValue, F32Ext, Hit, Reservoir, SpecularBrdf, Vec3Ext};

/// Reservoir for sampling indirect lightning.
///
/// See: [`Reservoir`].
#[derive(Clone, Copy, Default)]
pub struct IndirectReservoir {
    pub reservoir: Reservoir<IndirectReservoirSample>,
}

impl IndirectReservoir {
    pub fn new(sample: IndirectReservoirSample, p_hat: f32) -> Self {
        Self {
            reservoir: Reservoir::new(sample, p_hat),
        }
    }

    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.get_unchecked(4 * id) };
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
            *buffer.get_unchecked_mut(4 * id) = d0;
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
        self.radiance.luminance()
    }

    pub fn spatial_p_hat(&self, point: Vec3, normal: Vec3) -> f32 {
        self.temporal_p_hat() * self.direction(point).dot(normal).max(0.0)
    }

    pub fn direction(&self, point: Vec3) -> Vec3 {
        (self.sample_point - point).normalize()
    }

    pub fn cosine(&self, hit: &Hit) -> f32 {
        self.direction(hit.point).dot(hit.gbuffer.normal).max(0.0)
    }

    pub fn diffuse_brdf(&self, hit: &Hit) -> BrdfValue {
        BrdfValue {
            radiance: Vec3::ONE * (1.0 - hit.gbuffer.metallic),
            probability: PI,
        }
    }

    pub fn specular_brdf(&self, hit: &Hit) -> BrdfValue {
        let l = self.direction(hit.point);
        let v = -hit.direction;

        SpecularBrdf::new(&hit.gbuffer).evaluate(l, v)
    }

    pub fn is_within_specular_lobe_of(&self, hit: &Hit) -> bool {
        let l = self.direction(hit.point);
        let v = -hit.direction;

        SpecularBrdf::new(&hit.gbuffer).is_sample_within_lobe(l, v)
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
