mod direct;
mod ephemeral;
mod indirect;

pub use self::direct::*;
pub use self::ephemeral::*;
pub use self::indirect::*;
use crate::WhiteNoise;

/// Reservoir for sampling using ReSTIR
#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Reservoir<T> {
    /// Selected sample; might contain light id, its radiance etc.
    pub sample: T,

    /// Sum of the weights of seen samples.
    pub w_sum: f32,

    /// Number of seen samples¹.
    ///
    /// It's capped to a certain limit, depending on the reservoir's kind, over
    /// the temporal and spatial resampling passes.
    ///
    /// ¹ so technically kinda-sorta u32, but using f32 allows for convenient
    ///   things like `m *= 0.25;`
    pub m: f32,

    /// Reweighting factor.
    ///
    /// It's capped to a certain limit, depending on the reservoir's kind, over
    /// the temporal and spatial resampling passes.
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
            m: 1.0,
        }
    }

    pub fn add(
        &mut self,
        wnoise: &mut WhiteNoise,
        sample: T,
        weight: f32,
    ) -> bool {
        self.w_sum += weight;
        self.m += 1.0;

        if self.w_sum == 0.0 || wnoise.sample() <= weight / self.w_sum {
            self.sample = sample;
            true
        } else {
            false
        }
    }

    pub fn merge(
        &mut self,
        wnoise: &mut WhiteNoise,
        rhs: &Self,
        p_hat: f32,
    ) -> bool {
        if rhs.m <= 0.0 {
            return false;
        }

        self.m += rhs.m - 1.0;
        self.add(wnoise, rhs.sample, rhs.w * rhs.m * p_hat)
    }

    pub fn normalize(&mut self, p_hat: f32) {
        let t = self.m * p_hat;

        self.w = if t == 0.0 { 0.0 } else { self.w_sum / t };
    }

    pub fn clamp_m(&mut self, max: f32) {
        self.m = self.m.min(max);
    }
}
