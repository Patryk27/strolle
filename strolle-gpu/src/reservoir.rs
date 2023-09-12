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
    ///   things like `m_sum *= 0.25;`
    pub m_sum: f32,

    /// Reweighting factor, following the ReSTIR paper.
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
            m_sum: 1.0,
        }
    }

    pub fn add(
        &mut self,
        wnoise: &mut WhiteNoise,
        sample: T,
        weight: f32,
    ) -> bool {
        self.w_sum += weight;
        self.m_sum += 1.0;

        if wnoise.sample() <= weight / self.w_sum {
            self.sample = sample;
            true
        } else {
            false
        }
    }

    pub fn set(&mut self, sample: T, weight: f32) {
        self.w_sum += weight;
        self.m_sum += 1.0;
        self.sample = sample;
    }

    pub fn merge(
        &mut self,
        wnoise: &mut WhiteNoise,
        rhs: &Self,
        p_hat: f32,
    ) -> bool {
        // If the reservoir is empty, reject its sample as soon as possible.
        //
        // Note that it looks like the code below would do it anyway (since we
        // multiply by m_sum there), but the thing is that if both `self` *and*
        // `rhs` are empty reservoirs, without this explicit `if` here we would
        // merge `rhs` into `self` even if it doesn't actually contain any valid
        // sample.
        //
        // This comes up mostly (only?) for indirect lightning reservoirs which
        // can contain illegal samples (e.g. with zeroed-out normals) if the
        // camera is looking at the sky - and if we didn't handle those illegal
        // samples here, we could propagate those zeroed-out normals and other
        // funky numbers up to the spatial resampling pass which would then end
        // up generating NaN and INFs Jacobians: baaaad.
        if rhs.m_sum <= 0.0 {
            return false;
        }

        self.m_sum += rhs.m_sum - 1.0;
        self.add(wnoise, rhs.sample, rhs.w * rhs.m_sum * p_hat)
    }

    pub fn normalize(&mut self, p_hat: f32) {
        let t = self.m_sum * p_hat;

        self.w = if t == 0.0 { 0.0 } else { self.w_sum / t };
    }

    pub fn clamp_m(&mut self, max: f32) {
        self.m_sum = self.m_sum.min(max);
    }

    pub fn clamp_w(&mut self, max: f32) {
        self.w = self.w.min(max);
    }
}
