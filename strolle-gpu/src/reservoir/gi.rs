use core::ops::{Deref, DerefMut};

use glam::{Vec3, Vec4, Vec4Swizzles};
use spirv_std::arch::IndexUnchecked;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{
    DiffuseBrdf, F32Ext, Hit, Normal, Ray, Reservoir, SpecularBrdf, Vec3Ext,
};

#[derive(Clone, Copy, Default)]
pub struct GiReservoir {
    pub reservoir: Reservoir<GiSample>,
    pub confidence: f32,
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
                    pdf: d2.w,
                    rng: d3.w.to_bits(),
                    radiance: d0.xyz(),
                    v1_point: d1.xyz(),
                    v2_point: d2.xyz(),
                    v2_normal: Normal::decode(d3.xy()),
                },
                m: d0.w,
                w: d1.w,
            },
            confidence: d3.z,
        }
    }

    pub fn write(self, buffer: &mut [Vec4], id: usize) {
        let d0 = self.sample.radiance.extend(self.m);
        let d1 = self.sample.v1_point.extend(self.w);
        let d2 = self.sample.v2_point.extend(self.sample.pdf);

        let d3 = Normal::encode(self.sample.v2_normal)
            .extend(self.confidence)
            .extend(f32::from_bits(self.sample.rng));

        unsafe {
            *buffer.index_unchecked_mut(4 * id) = d0;
            *buffer.index_unchecked_mut(4 * id + 1) = d1;
            *buffer.index_unchecked_mut(4 * id + 2) = d2;
            *buffer.index_unchecked_mut(4 * id + 3) = d3;
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
    pub pdf: f32,
    pub rng: u32,
    pub radiance: Vec3,
    pub v1_point: Vec3, // TODO can't we simply fetch it from gbuffer instead?
    pub v2_point: Vec3,
    pub v2_normal: Vec3,
}

impl GiSample {
    pub fn exists(self) -> bool {
        self.v2_point != Default::default()
    }

    pub fn pdf(self, mut hit: Hit) -> f32 {
        if !self.exists() {
            return 0.0;
        }

        hit.gbuffer.base_color = Vec4::ONE;

        let diff_brdf = self.diff_brdf(hit).luma();

        // TODO cut off specular tail, to improve resampling near edges
        let spec_brdf = self.spec_brdf(hit).luma();

        // TODO use something simpler for the PDF
        self.radiance.luma() * self.cosine(hit) * (diff_brdf + spec_brdf)
    }

    pub fn ray(self, hit_point: Vec3) -> Ray {
        Ray::new(hit_point, self.dir(hit_point))
            .with_len(self.v2_point.distance(hit_point) - 0.01)
    }

    pub fn dir(self, point: Vec3) -> Vec3 {
        (self.v2_point - point).normalize()
    }

    pub fn cosine(self, hit: Hit) -> f32 {
        self.dir(hit.point).dot(hit.gbuffer.normal).max(0.0)
    }

    pub fn diff_brdf(self, hit: Hit) -> Vec3 {
        DiffuseBrdf::new(hit.gbuffer).eval()
    }

    pub fn spec_brdf(self, hit: Hit) -> Vec3 {
        SpecularBrdf::new(hit.gbuffer).eval(self.dir(hit.point), -hit.dir)
    }

    pub fn jacobian(self, new_hit_point: Vec3) -> f32 {
        if !self.exists() {
            return 1.0;
        }

        let (new_dist, new_cos) = self.partial_jacobian(new_hit_point);
        let (old_dist, old_cos) = self.partial_jacobian(self.v1_point);

        let x = new_cos * old_dist * old_dist;
        let y = old_cos * new_dist * new_dist;

        if y == 0.0 {
            0.0
        } else {
            x / y
        }
    }

    fn partial_jacobian(self, hit_point: Vec3) -> (f32, f32) {
        let vec = hit_point - self.v2_point;
        let dist = vec.length();
        let cos = self.v2_normal.dot(vec / dist).saturate();

        (dist, cos)
    }
}
