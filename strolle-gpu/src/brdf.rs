use core::f32::consts::PI;

use glam::{Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{F32Ext, GBufferEntry, WhiteNoise};

#[derive(Clone, Copy)]
pub struct DiffuseBrdf {
    gbuffer: GBufferEntry,
}

impl DiffuseBrdf {
    pub fn new(gbuffer: GBufferEntry) -> Self {
        Self { gbuffer }
    }

    // TODO separate eval_luma()
    pub fn eval(self) -> Vec3 {
        let Self { gbuffer } = self;

        gbuffer.base_color.xyz() * (1.0 - gbuffer.metallic) / PI
    }

    pub fn sample(self, wnoise: &mut WhiteNoise) -> BrdfSample {
        BrdfSample {
            dir: wnoise.sample_hemisphere(self.gbuffer.normal),
            pdf: 1.0 / PI,
            radiance: self.eval(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpecularBrdf {
    gbuffer: GBufferEntry,
}

impl SpecularBrdf {
    pub fn new(gbuffer: GBufferEntry) -> Self {
        Self { gbuffer }
    }

    // TODO separate eval_luma()
    pub fn eval(self, l: Vec3, v: Vec3) -> Vec3 {
        let Self { gbuffer } = self;

        if gbuffer.metallic <= 0.0 {
            return Vec3::ZERO;
        }

        let a = gbuffer.clamped_roughness();
        let n = gbuffer.normal;
        let h = (l + v).normalize();
        let n_dot_l = n.dot(l).saturate();
        let n_dot_h = n.dot(h).saturate();
        let l_dot_h = l.dot(h).saturate();
        let n_dot_v = n.dot(v).saturate();

        if n_dot_l <= 0.0 || n_dot_v <= 0.0 {
            return Vec3::ZERO;
        }

        let d = ggx_distribution(n_dot_h, a);
        let g = ggx_schlick_masking_term(n_dot_l, n_dot_v, a);

        let f = {
            let f0 = 0.16
                * gbuffer.reflectance
                * gbuffer.reflectance
                * (1.0 - gbuffer.metallic)
                + gbuffer.base_color.xyz() * gbuffer.metallic;

            ggx_schlick_fresnel(f0, l_dot_h)
        };

        d * g * f / (4.0 * n_dot_l * n_dot_v)
    }

    // TODO implement VNDF
    pub fn sample(self, wnoise: &mut WhiteNoise, v: Vec3) -> BrdfSample {
        let Self { gbuffer } = self;

        let r0 = wnoise.sample();
        let r1 = wnoise.sample();

        let a = gbuffer.clamped_roughness();
        let n = gbuffer.normal;
        let a2 = a.sqr();
        let (b, t) = n.any_orthonormal_pair();

        let cos_theta = 0.0f32.max((1.0 - r0) / ((a2 - 1.0) * r0 + 1.0)).sqrt();
        let sin_theta = 0.0f32.max(1.0 - cos_theta * cos_theta).sqrt();

        let phi = r1 * PI * 2.0;

        let h = t * (sin_theta * phi.cos())
            + b * (sin_theta * phi.sin())
            + n * cos_theta;

        let n_dot_h = n.dot(h).saturate();
        let h_dot_v = h.dot(v).saturate();

        let dir = (2.0 * h_dot_v * h - v).normalize();
        let pdf = ggx_distribution(n_dot_h, a) * n_dot_h / (4.0 * h_dot_v);

        BrdfSample {
            dir,
            pdf,
            radiance: self.eval(dir, v),
        }
    }
}

pub struct LayeredBrdf {
    gbuffer: GBufferEntry,
}

impl LayeredBrdf {
    pub fn new(gbuffer: GBufferEntry) -> Self {
        Self { gbuffer }
    }

    pub fn sample(self, wnoise: &mut WhiteNoise, l: Vec3) -> BrdfSample {
        let Self { gbuffer } = self;
        let mut sample;

        if wnoise.sample() < gbuffer.metallic {
            sample = SpecularBrdf::new(gbuffer).sample(wnoise, l);
            sample.pdf /= gbuffer.metallic;
        } else {
            sample = DiffuseBrdf::new(gbuffer).sample(wnoise);
            sample.pdf /= 1.0 - gbuffer.metallic;
        }

        sample
    }
}

#[derive(Clone, Copy)]
pub struct BrdfSample {
    pub dir: Vec3,
    pub pdf: f32,
    pub radiance: Vec3,
}

impl BrdfSample {
    pub fn invalid() -> Self {
        Self {
            dir: Default::default(),
            pdf: Default::default(),
            radiance: Default::default(),
        }
    }

    pub fn is_invalid(self) -> bool {
        self.pdf == 0.0
    }
}

fn ggx_schlick_fresnel(f0: Vec3, l_dot_h: f32) -> Vec3 {
    let f90 = f0.dot(Vec3::splat(50.0 * 0.33)).saturate();

    f_schlick_vec(f0, f90, l_dot_h)
}

fn ggx_distribution(n_dot_h: f32, roughness: f32) -> f32 {
    let a2 = roughness * roughness;
    let d = (n_dot_h * a2 - n_dot_h) * n_dot_h + 1.0;

    a2 / (PI * d * d)
}

fn ggx_schlick_masking_term(n_dot_l: f32, n_dot_v: f32, roughness: f32) -> f32 {
    let k = roughness * roughness / 2.0;

    let g_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let g_l = n_dot_l / (n_dot_l * (1.0 - k) + k);

    g_v * g_l
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_dot_h: f32) -> Vec3 {
    f0 + (f90 - f0) * (1.0 - v_dot_h).max(0.001).powf(5.0)
}
