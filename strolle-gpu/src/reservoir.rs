mod direct;
mod ephemeral;
mod indirect;

pub use self::direct::*;
pub use self::ephemeral::*;
pub use self::indirect::*;
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

        if wnoise.sample() * self.w <= weight {
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
        pdf: f32,
    ) -> bool {
        if rhs.m <= 0.0 {
            return false;
        }

        self.m += rhs.m - 1.0;
        self.update(wnoise, rhs.sample, rhs.w * rhs.m * pdf)
    }

    pub fn normalize(&mut self, pdf: f32) {
        let t = self.m * pdf;

        self.w = if t == 0.0 { 0.0 } else { self.w / t };
    }

    pub fn normalize_ex(&mut self, pdf: f32, norm_num: f32, norm_denom: f32) {
        let denom = pdf * norm_denom;

        self.w = if denom == 0.0 {
            0.0
        } else {
            (self.w * norm_num) / denom
        };
    }

    pub fn clamp_m(&mut self, max: f32) {
        self.m = self.m.min(max);
    }
}
