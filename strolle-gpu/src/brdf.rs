use core::f32::consts::PI;

use glam::{vec3, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::{F32Ext, GBufferEntry, Hit, Vec3Ext, WhiteNoise};

#[derive(Clone, Copy)]
pub struct DiffuseBrdf<'a> {
    gbuffer: &'a GBufferEntry,
}

impl<'a> DiffuseBrdf<'a> {
    pub fn new(gbuffer: &'a GBufferEntry) -> Self {
        Self { gbuffer }
    }

    pub fn evaluate(self, v: Vec3, l: Vec3) -> BrdfValue {
        let Self { gbuffer } = self;

        let n = gbuffer.normal;
        let h = (l + v).normalize();
        let n_dot_v = n.dot(v).max(0.0001);
        let n_dot_l = n.dot(l).saturate();
        let l_dot_h = l.dot(h).saturate();

        let radiance = gbuffer.base_color.xyz()
            * fd_burley(gbuffer.clamped_roughness(), n_dot_v, n_dot_l, l_dot_h)
            * (1.0 - gbuffer.metallic);

        BrdfValue {
            radiance,
            probability: PI,
        }
    }

    pub fn sample(self, wnoise: &mut WhiteNoise) -> BrdfSample {
        BrdfSample {
            direction: wnoise.sample_hemisphere(self.gbuffer.normal),
            throughput: self.gbuffer.base_color.xyz()
                * (1.0 - self.gbuffer.metallic),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpecularBrdf<'a> {
    gbuffer: &'a GBufferEntry,
}

impl<'a> SpecularBrdf<'a> {
    pub fn new(gbuffer: &'a GBufferEntry) -> Self {
        Self { gbuffer }
    }

    pub fn evaluate(self, v: Vec3, l: Vec3) -> BrdfValue {
        let Self { gbuffer } = self;

        let roughness = gbuffer.clamped_roughness();
        let n = gbuffer.normal;
        let h = (v + l).normalize();
        let n_dot_l = n.dot(l).saturate();
        let n_dot_h = n.dot(h).saturate();
        let l_dot_h = l.dot(h).saturate();
        let n_dot_v = n.dot(v).saturate();

        if n_dot_l <= 0.0 || n_dot_v <= 0.0 {
            return BrdfValue::default();
        }

        let d = ggx_distribution(n_dot_h, roughness);
        let g = ggx_schlick_masking_term(n_dot_l, n_dot_v, roughness);

        let f = {
            let f0 = 0.16
                * gbuffer.reflectance
                * gbuffer.reflectance
                * (1.0 - gbuffer.metallic)
                + gbuffer.base_color.xyz() * gbuffer.metallic;

            ggx_schlick_fresnel(f0, l_dot_h)
        };

        let radiance = d * g * f / (4.0 * n_dot_l * n_dot_v);

        let probability = {
            let a2 = roughness.sqr();

            let g1_mod = n_dot_v
                + ((n_dot_v - a2 * n_dot_v) * n_dot_v + a2).saturate().sqrt();

            let g1_mod = if g1_mod <= 0.0 { 0.0 } else { 1.0 / g1_mod };

            d * g1_mod * 0.5
        };

        BrdfValue {
            radiance,
            probability,
        }
    }

    pub fn sample(self, wnoise: &mut WhiteNoise, hit: Hit) -> BrdfSample {
        fn to_world_coords(x: Vec3, y: Vec3, z: Vec3, v: Vec3) -> Vec3 {
            v.x * x + v.y * y + v.z * z
        }

        fn to_local_coords(x: Vec3, y: Vec3, z: Vec3, v: Vec3) -> Vec3 {
            vec3(v.dot(x), v.dot(y), v.dot(z))
        }

        fn ggx(
            v_local: Vec3,
            roughness: f32,
            sample1: f32,
            sample2: f32,
        ) -> Vec3 {
            let v_h =
                vec3(roughness * v_local.x, roughness * v_local.y, v_local.z)
                    .normalize();

            let len = v_h.x * v_h.x + v_h.y * v_h.y;

            let tt1 = if len > 0.0 {
                vec3(-v_h.y, v_h.x, 0.0) * (1.0 / len.sqrt())
            } else {
                vec3(1.0, 0.0, 0.0)
            };

            let tt2 = v_h.cross(tt1);

            let r = sample1.sqrt();
            let phi = 2.0 * PI * sample2;
            let t1 = r * phi.cos();
            let t2 = r * phi.sin();
            let s = 0.5 * (1.0 + v_h.z);
            let t2 = (1.0 - s) * (1.0 - t1 * t1).sqrt() + s * t2;

            let n_h = t1 * tt1
                + t2 * tt2
                + 0.0f32.max(1.0 - t1 * t1 - t2 * t2).sqrt() * v_h;

            vec3(roughness * n_h.x, roughness * n_h.y, 0.0f32.max(n_h.z))
                .normalize()
        }

        let n = hit.gbuffer.normal;
        let v = -hit.direction;
        let (t, b) = n.any_orthonormal_pair();

        let mut sample_idx = 0;

        loop {
            sample_idx += 1;

            let v_local = to_local_coords(t, b, n, v);

            let mut h_local = ggx(
                v_local,
                self.gbuffer.roughness,
                wnoise.sample(),
                wnoise.sample(),
            );

            if h_local.z < 0.0 {
                h_local = -h_local;
            }

            let h = to_world_coords(t, b, n, h_local);
            let l = (-v).reflect(h);
            let n_dot_l = n.dot(l);
            let n_dot_v = n.dot(v);

            if n_dot_l > 0.0 && n_dot_v > 0.0 {
                let value = self.evaluate(v, l);

                if value.probability > 0.01 {
                    break BrdfSample {
                        direction: l,
                        throughput: value.radiance / value.probability,
                    };
                }
            }

            if sample_idx >= 16 {
                return BrdfSample::invalid();
            }
        }
    }

    pub fn is_sample_within_lobe(&self, v: Vec3, l: Vec3) -> bool {
        let Self { gbuffer } = self;

        let roughness = gbuffer.clamped_roughness();
        let n = gbuffer.normal;
        let h = (l + v).normalize();

        ggx_distribution(n.dot(h), roughness) > 0.1
    }
}

pub struct LayeredBrdf;

impl LayeredBrdf {
    pub fn sample(wnoise: &mut WhiteNoise, hit: Hit) -> BrdfSample {
        let mut sample = if wnoise.sample() <= 0.5 {
            DiffuseBrdf::new(&hit.gbuffer).sample(wnoise)
        } else {
            SpecularBrdf::new(&hit.gbuffer).sample(wnoise, hit)
        };

        sample.throughput *= 2.0;
        sample
    }
}

#[derive(Clone, Copy, Default)]
pub struct BrdfValue {
    pub radiance: Vec3,
    pub probability: f32,
}

#[derive(Clone, Copy)]
pub struct BrdfSample {
    pub direction: Vec3,
    pub throughput: Vec3,
}

impl BrdfSample {
    pub fn invalid() -> Self {
        Self {
            direction: Default::default(),
            throughput: Default::default(),
        }
    }

    pub fn is_invalid(&self) -> bool {
        self.direction == Default::default()
    }
}

fn fd_burley(roughness: f32, n_dot_v: f32, n_dot_l: f32, l_dot_h: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * l_dot_h * l_dot_h;
    let light_scatter = f_schlick(1.0, f90, n_dot_l);
    let view_scatter = f_schlick(1.0, f90, n_dot_v);

    light_scatter * view_scatter * (1.0 / PI)
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

fn f_schlick(f0: f32, f90: f32, v_dot_h: f32) -> f32 {
    f0 + (f90 - f0) * (1.0 - v_dot_h).max(0.001).powf(5.0)
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_dot_h: f32) -> Vec3 {
    f0 + (f90 - f0) * (1.0 - v_dot_h).max(0.001).powf(5.0)
}
