use core::f32::consts::PI;
use core::ops::{Deref, DerefMut};

use glam::{Vec3, Vec4, Vec4Swizzles};
use spirv_std::arch::IndexUnchecked;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{BrdfValue, F32Ext, Hit, Normal, Reservoir, SpecularBrdf, Vec3Ext};

#[derive(Clone, Copy, Default)]
pub struct GiReservoir {
    pub reservoir: Reservoir<GiSample>,
}

impl GiReservoir {
    pub fn read(buffer: &[Vec4], id: usize) -> Self {
        let d0 = unsafe { *buffer.index_unchecked(4 * id) };
        let d1 = unsafe { *buffer.index_unchecked(4 * id + 1) };
        let d2 = unsafe { *buffer.index_unchecked(4 * id + 2) };
        let d3 = unsafe { *buffer.index_unchecked(4 * id + 3) };

        Self {
            reservoir: Reservoir {
                sample: GiSample {
                    radiance: d0.xyz(),
                    v1_point: d1.xyz(),
                    v2_point: d2.xyz(),
                    v2_normal: Normal::decode(d3.xy()),
                    frame: d2.w.to_bits(),
                },
                m: d0.w,
                w: d1.w,
            },
        }
    }

    pub fn write(&self, buffer: &mut [Vec4], id: usize) {
        let d0 = self.sample.radiance.extend(self.m);
        let d1 = self.sample.v1_point.extend(self.w);

        let d2 = self
            .sample
            .v2_point
            .extend(f32::from_bits(self.sample.frame));

        let d3 = Normal::encode(self.sample.v2_normal)
            .extend(0.0)
            .extend(0.0);

        unsafe {
            *buffer.index_unchecked_mut(4 * id) = d0;
            *buffer.index_unchecked_mut(4 * id + 1) = d1;
            *buffer.index_unchecked_mut(4 * id + 2) = d2;
            *buffer.index_unchecked_mut(4 * id + 3) = d3;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sample.frame == 0
    }
}

impl Deref for GiReservoir {
    type Target = Reservoir<GiSample>;

    fn deref(&self) -> &Self::Target {
        &self.reservoir
    }
}

impl DerefMut for GiReservoir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reservoir
    }
}

#[derive(Clone, Copy, Default)]
pub struct GiSample {
    pub radiance: Vec3,
    pub v1_point: Vec3,
    pub v2_point: Vec3,
    pub v2_normal: Vec3,
    pub frame: u32,
}

impl GiSample {
    pub fn specular_pdf(&self) -> f32 {
        self.radiance.luminance()
    }

    pub fn diffuse_pdf(&self, hit_point: Vec3, hit_normal: Vec3) -> f32 {
        self.radiance.luminance()
            * self.direction(hit_point).dot(hit_normal).max(0.0)
    }

    pub fn direction(&self, point: Vec3) -> Vec3 {
        (self.v2_point - point).normalize()
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

        let (orig_distance, orig_cosine) = self.partial_jacobian(self.v1_point);

        let x = new_cosine * orig_distance * orig_distance;
        let y = orig_cosine * new_distance * new_distance;

        if y == 0.0 {
            0.0
        } else {
            x / y
        }
    }

    fn partial_jacobian(&self, hit_point: Vec3) -> (f32, f32) {
        let vec = hit_point - self.v2_point;
        let distance = vec.length();
        let cosine = self.v2_normal.dot(vec / distance).saturate();

        (distance, cosine)
    }
}
