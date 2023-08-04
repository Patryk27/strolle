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

    pub fn eval(self, l: Vec3, v: Vec3, n: Vec3, n_o_l: f32) -> Vec3 {
        let h = (l + v).normalize();
        let n_o_v = n.dot(v).max(0.0001);
        let l_o_h = l.dot(h).saturate();

        self.gbuffer.base_color.xyz()
            * fd_burley(self.gbuffer.clamped_roughness(), n_o_v, n_o_l, l_o_h)
            * (1.0 - self.gbuffer.metallic)
    }

    pub fn sample(self, wnoise: &mut WhiteNoise) -> Vec3 {
        wnoise.sample_hemisphere(self.gbuffer.normal)
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

    pub fn eval(self, n_o_v: f32, n_o_l: f32, n_o_h: f32, l_o_h: f32) -> Vec3 {
        let f0 = 0.16
            * self.gbuffer.reflectance
            * self.gbuffer.reflectance
            * (1.0 - self.gbuffer.metallic)
            + self.gbuffer.base_color.xyz() * self.gbuffer.metallic;

        let d = d_ggx(self.gbuffer.clamped_roughness(), n_o_h);

        let v = v_smith_ggx_correlated(
            self.gbuffer.clamped_roughness(),
            n_o_v,
            n_o_l,
        );

        let f = fresnel(f0, l_o_h);

        d * v * f
    }

    pub fn eval_f(self, l: Vec3, v: Vec3, n: Vec3) -> f32 {
        let h = (l + v).normalize();
        let n_o_l = n.dot(l).saturate();
        let n_o_v = n.dot(v).saturate();
        let n_o_h = n.dot(h).saturate();

        let d = d_ggx(self.gbuffer.clamped_roughness(), n_o_h);

        let v = v_smith_ggx_correlated(
            self.gbuffer.clamped_roughness(),
            n_o_v,
            n_o_l,
        );

        d * v
    }

    pub fn sample(self, wnoise: &mut WhiteNoise, hit: Hit) -> Vec3 {
        fn to_world(x: Vec3, y: Vec3, z: Vec3, v: Vec3) -> Vec3 {
            v.x * x + v.y * y + v.z * z
        }

        fn to_local(x: Vec3, y: Vec3, z: Vec3, v: Vec3) -> Vec3 {
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

        let mut sample = 0;

        loop {
            sample += 1;

            let v_local = to_local(t, b, n, v);

            let mut h = ggx(
                v_local,
                self.gbuffer.roughness,
                wnoise.sample(),
                wnoise.sample(),
            );

            if h.z < 0.0 {
                h = -h;
            }

            h = to_world(t, b, n, h);

            let l = (-v).reflect(h);
            let n_o_l = n.dot(l);
            let n_o_v = n.dot(v);

            if (n_o_l > 0.0 && n_o_v > 0.0) || (sample >= 16) {
                break l;
            }
        }
    }
}

pub struct LayeredBrdf;

impl LayeredBrdf {
    pub fn sample(wnoise: &mut WhiteNoise, hit: Hit) -> Vec3 {
        if wnoise.sample() <= 0.5 {
            DiffuseBrdf::new(&hit.gbuffer).sample(wnoise)
        } else {
            SpecularBrdf::new(&hit.gbuffer).sample(wnoise, hit)
        }
    }
}

fn fd_burley(roughness: f32, n_o_v: f32, n_o_l: f32, l_o_h: f32) -> f32 {
    let f90 = 0.5 + 2.0 * roughness * l_o_h * l_o_h;
    let light_scatter = f_schlick(1.0, f90, n_o_l);
    let view_scatter = f_schlick(1.0, f90, n_o_v);

    light_scatter * view_scatter * (1.0 / PI)
}

fn d_ggx(roughness: f32, n_o_h: f32) -> f32 {
    let one_minus_noh_squared = 1.0 - n_o_h * n_o_h;
    let a = n_o_h * roughness;
    let k = roughness / (one_minus_noh_squared + a * a);

    k * k * (1.0 / PI)
}

fn v_smith_ggx_correlated(roughness: f32, n_o_v: f32, n_o_l: f32) -> f32 {
    let a2 = roughness * roughness;
    let lambda_v = n_o_l * ((n_o_v - a2 * n_o_v) * n_o_v + a2).sqrt();
    let lambda_l = n_o_v * ((n_o_l - a2 * n_o_l) * n_o_l + a2).sqrt();

    0.5 / (lambda_v + lambda_l)
}

fn fresnel(f0: Vec3, l_o_h: f32) -> Vec3 {
    let f90 = f0.dot(Vec3::splat(50.0 * 0.33)).saturate();

    f_schlick_vec(f0, f90, l_o_h)
}

fn f_schlick(f0: f32, f90: f32, v_o_h: f32) -> f32 {
    f0 + (f90 - f0) * (1.0 - v_o_h).max(0.001).powf(5.0)
}

fn f_schlick_vec(f0: Vec3, f90: f32, v_o_h: f32) -> Vec3 {
    f0 + (f90 - f0) * (1.0 - v_o_h).max(0.001).powf(5.0)
}
