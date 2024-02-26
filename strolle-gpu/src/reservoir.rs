mod di;
mod ephemeral;
mod gi;
mod mis;

pub use self::di::*;
pub use self::ephemeral::*;
pub use self::gi::*;
pub use self::mis::*;
use crate::WhiteNoise;

#[derive(Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct Reservoir<T> {
    pub sample: T,
    pub m: f32,
    pub w: f32,
}

impl<T> Reservoir<T>
where
    T: Clone + Copy,
{
    pub fn update(
        &mut self,
        wnoise: &mut WhiteNoise,
        sample: T,
        weight: f32,
    ) -> bool {
        self.m += 1.0;
        self.w += weight;

        if wnoise.sample() * self.w < weight {
            self.sample = sample;
            true
        } else {
            false
        }
    }

    pub fn merge(
        &mut self,
        wnoise: &mut WhiteNoise,
        sample: &Self,
        pdf: f32,
    ) -> bool {
        if sample.m <= 0.0 {
            return false;
        }

        self.m += sample.m - 1.0;
        self.update(wnoise, sample.sample, sample.w * sample.m * pdf)
    }

    pub fn clamp_m(&mut self, max: f32) {
        self.m = self.m.min(max);
    }

    pub fn clamp_w(&mut self, max: f32) {
        self.w = self.w.min(max);
    }

    pub fn norm(&mut self, pdf: f32, norm_num: f32, norm_denom: f32) {
        let denom = pdf * norm_denom;

        self.w = if denom == 0.0 {
            0.0
        } else {
            (self.w * norm_num) / denom
        };
    }

    pub fn norm_avg(&mut self, pdf: f32) {
        self.norm(pdf, 1.0, self.m);
    }

    pub fn norm_mis(&mut self, pdf: f32) {
        self.norm(pdf, 1.0, 1.0);
    }
}
