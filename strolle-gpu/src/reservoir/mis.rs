#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{DiReservoir, F32Ext, GiReservoir, Hit, LightsView};

/// Helper for calculating factors for multiple importance sampling.
///
/// Code assumes we'd like to merge two samples, where `lhs` is the canonical
/// one and `rhs` is the neighbour (which is an important distinction, because
/// we implement the defensive variant here).
#[derive(Clone, Copy, Default)]
pub struct Mis {
    /// Confidence weight for the canonical sample
    pub lhs_m: f32,

    /// Confidence weight for the neighbour sample
    pub rhs_m: f32,

    /// Jacobian determinant of the neighbour sample; 1.0 if not applicable
    pub rhs_jacobian: f32,

    /// `p_lhs(lhs)`, i.e. probability of lhs's sample on lhs's pixel
    pub lhs_lhs_pdf: f32,

    /// `p_lhs(rhs)`, i.e. probability of lhs's sample on rhs's pixel
    pub lhs_rhs_pdf: f32,

    /// `p_rhs(lhs)`, i.e. probability of rhs's sample on lhs's pixel
    pub rhs_lhs_pdf: f32,

    /// `p_rhs(rhs)`, i.e. probability of rhs's sample on rhs's pixel
    pub rhs_rhs_pdf: f32,
}

impl Mis {
    pub fn di_temporal(
        lights: LightsView,
        lhs: DiReservoir,
        lhs_hit: Hit,
        rhs: DiReservoir,
        rhs_hit: Hit,
        rhs_killed: bool,
    ) -> Self {
        let lhs_rhs_pdf = if (lhs.m > 0.0) & rhs_hit.is_some() {
            lhs.sample.pdf_prev(lights, rhs_hit)
        } else {
            0.0
        };

        let rhs_lhs_pdf = if (rhs.m > 0.0) & !rhs_killed {
            rhs.sample.pdf(lights, lhs_hit)
        } else {
            0.0
        };

        Self {
            lhs_m: lhs.m,
            rhs_m: rhs.m,
            rhs_jacobian: 1.0,
            lhs_lhs_pdf: lhs.sample.pdf,
            lhs_rhs_pdf,
            rhs_lhs_pdf,
            rhs_rhs_pdf: rhs.sample.pdf,
        }
    }

    pub fn gi_temporal(
        lhs: GiReservoir,
        lhs_hit: Hit,
        rhs: GiReservoir,
        rhs_hit: Hit,
    ) -> Self {
        let lhs_rhs_pdf = if (lhs.m > 0.0) & rhs_hit.is_some() {
            lhs.sample.pdf(rhs_hit)
        } else {
            0.0
        };

        let rhs_lhs_pdf = if rhs.m > 0.0 {
            rhs.sample.pdf(lhs_hit)
        } else {
            0.0
        };

        Self {
            lhs_m: lhs.m,
            rhs_m: rhs.m,
            rhs_jacobian: 1.0,
            lhs_lhs_pdf: lhs.sample.pdf,
            lhs_rhs_pdf,
            rhs_lhs_pdf,
            rhs_rhs_pdf: rhs.sample.pdf,
        }
    }

    pub fn eval(self) -> MisResult {
        fn mis(x: f32, y: f32) -> f32 {
            let sum = x + y;

            if sum == 0.0 {
                0.0
            } else {
                x / sum
            }
        }

        fn m(q0: f32, q1: f32) -> f32 {
            if q0 <= 0.0 {
                1.0
            } else {
                (q1 / q0).min(1.0).powf(8.0).saturate()
            }
        }

        let m = self.rhs_m
            * m(self.rhs_rhs_pdf, self.rhs_lhs_pdf)
                .min(m(self.lhs_rhs_pdf, self.lhs_lhs_pdf));

        // We implement the defensive variant, giving some extra score to the
        // canonical sample - this comes *very* handy in reducing variance in
        // penumbra regions for direct lighting when doing spatial resampling
        let t = mis(self.lhs_m, self.rhs_m);

        let lhs_mis = t
            + (1.0 - t)
                * mis(
                    self.lhs_m * self.lhs_lhs_pdf,
                    self.rhs_m * self.lhs_rhs_pdf,
                );

        let rhs_mis = (1.0 - t)
            * mis(
                self.rhs_m * self.rhs_rhs_pdf * self.rhs_jacobian,
                self.lhs_m * self.rhs_lhs_pdf,
            );

        MisResult {
            m,
            lhs_pdf: self.lhs_lhs_pdf,
            lhs_mis,
            rhs_pdf: self.rhs_lhs_pdf,
            rhs_mis,
        }
    }
}

#[derive(Clone, Copy, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct MisResult {
    pub m: f32,
    pub lhs_pdf: f32,
    pub lhs_mis: f32,
    pub rhs_pdf: f32,
    pub rhs_mis: f32,
}
