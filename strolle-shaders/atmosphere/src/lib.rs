//! This pass generates lookup textures used to sample sky; this allows for
//! rendering a sky in real-time without requiring any expensive GPU.
//!
//! Thanks to:
//!
//! - https://www.shadertoy.com/view/slSXRW
//!   (Production Sky Rendering by AndrewHelmer)
//!
//! - https://github.com/sebh/UnrealEngineSkyAtmosphere
//!
//! Original license:
//!
//! ```text
//! MIT License
//!
//! Copyright (c) 2020 Epic Games, Inc.
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```

#![no_std]

use strolle_gpu::prelude::*;

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main_generate_transmittance_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] out: TexRgba16f,
) {
    let global_id = global_id.xy();

    let uv = global_id.as_vec2()
        / Atmosphere::TRANSMITTANCE_LUT_RESOLUTION.as_vec2();

    let sun_cos_theta = 2.0 * uv.x - 1.0;
    let sun_theta = sun_cos_theta.clamp(-1.0, 1.0).acos();

    let height = lerp(
        Atmosphere::GROUND_RADIUS_MM,
        Atmosphere::ATMOSPHERE_RADIUS_MM,
        uv.y,
    );

    let pos = vec3(0.0, height, 0.0);
    let sun_dir = vec3(0.0, sun_cos_theta, -sun_theta.sin()).normalize();
    let out_val = eval_transmittance(pos, sun_dir);

    unsafe {
        out.write(global_id, out_val.extend(1.0));
    }
}

fn eval_transmittance(pos: Vec3, sun_dir: Vec3) -> Vec3 {
    if Ray::new(pos, sun_dir).intersect_sphere(Atmosphere::GROUND_RADIUS_MM)
        > 0.0
    {
        return Default::default();
    }

    let atmosphere_dist = Ray::new(pos, sun_dir)
        .intersect_sphere(Atmosphere::ATMOSPHERE_RADIUS_MM);

    let mut t = 0.0;
    let mut transmittance = Vec3::splat(1.0);
    let mut i = 0.0;

    while i < Atmosphere::TRANSMITTANCE_LUT_STEPS {
        let new_t =
            ((i + 0.3) / Atmosphere::TRANSMITTANCE_LUT_STEPS) * atmosphere_dist;

        let dt = new_t - t;

        t = new_t;

        let new_pos = pos + t * sun_dir;
        let (_, _, extinction) = get_scattering_values(new_pos);

        transmittance *= (-dt * extinction).exp();
        i += 1.0;
    }

    transmittance
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main_generate_scattering_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 1)]
    transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 2)] out: TexRgba16f,
) {
    let global_id = global_id.xy();

    let uv =
        global_id.as_vec2() / Atmosphere::SCATTERING_LUT_RESOLUTION.as_vec2();

    let sun_cos_theta = 2.0 * uv.x - 1.0;
    let sun_theta = sun_cos_theta.clamp(-1.0, 1.0).acos();

    let height = lerp(
        Atmosphere::GROUND_RADIUS_MM,
        Atmosphere::ATMOSPHERE_RADIUS_MM,
        uv.y,
    );

    let pos = vec3(0.0, height, 0.0);
    let sun_dir = vec3(0.0, sun_cos_theta, -sun_theta.sin()).normalize();

    let (lum, f_ms) = eval_scattering(
        transmittance_lut_tex,
        transmittance_lut_sampler,
        pos,
        sun_dir,
    );

    let out_val = lum / (1.0 - f_ms);

    let global_id = uvec2(
        global_id.x,
        Atmosphere::SCATTERING_LUT_RESOLUTION.y - global_id.y + 1,
    );

    unsafe {
        out.write(global_id, out_val.extend(1.0));
    }
}

fn eval_scattering(
    transmittance_lut_tex: Tex,
    transmittance_lut_sampler: &Sampler,
    pos: Vec3,
    sun_dir: Vec3,
) -> (Vec3, Vec3) {
    let mut lum_total = Vec3::default();
    let mut fms = Vec3::default();

    let inv_samples = 1.0
        / ((Atmosphere::SCATTERING_LUT_SAMPLES_SQRT
            * Atmosphere::SCATTERING_LUT_SAMPLES_SQRT) as f32);

    let mut i = 0;

    while i < Atmosphere::SCATTERING_LUT_SAMPLES_SQRT {
        let mut j = 0;

        while j < Atmosphere::SCATTERING_LUT_SAMPLES_SQRT {
            let theta = PI * ((i as f32) + 0.5)
                / (Atmosphere::SCATTERING_LUT_SAMPLES_SQRT as f32);

            let phi = (1.0
                - 2.0 * ((j as f32) + 0.5)
                    / (Atmosphere::SCATTERING_LUT_SAMPLES_SQRT as f32))
                .clamp(-1.0, 1.0)
                .acos();

            let ray_dir = get_spherical_dir(theta, phi);

            let atmosphere_distance = Ray::new(pos, ray_dir)
                .intersect_sphere(Atmosphere::ATMOSPHERE_RADIUS_MM);

            let ground_distance = Ray::new(pos, ray_dir)
                .intersect_sphere(Atmosphere::GROUND_RADIUS_MM);

            let t_max = if ground_distance > 0.0 {
                ground_distance
            } else {
                atmosphere_distance
            };

            let cos_theta = ray_dir.dot(sun_dir);
            let mie_phase_value = get_mie_phase(cos_theta);
            let rayleigh_phase_value = get_rayleigh_phase(-cos_theta);

            let mut lum = Vec3::default();
            let mut lum_factor = Vec3::default();
            let mut transmittance = Vec3::splat(1.0);

            let mut t = 0.0;
            let mut step_i = 0.0;

            while step_i < Atmosphere::SCATTERING_LUT_STEPS {
                let new_t =
                    ((step_i + 0.3) / Atmosphere::SCATTERING_LUT_STEPS) * t_max;

                let dt = new_t - t;

                t = new_t;

                let new_pos = pos + t * ray_dir;

                let (rayleigh_scattering, mie_scattering, extinction) =
                    get_scattering_values(new_pos);

                let sample_transmittance = (-dt * extinction).exp();

                let scattering_no_phase = rayleigh_scattering + mie_scattering;

                let scattering_f = (scattering_no_phase
                    - scattering_no_phase * sample_transmittance)
                    / extinction;

                lum_factor += transmittance * scattering_f;

                let sun_transmittance = Atmosphere::sample_lut(
                    transmittance_lut_tex,
                    transmittance_lut_sampler,
                    new_pos,
                    sun_dir,
                );

                let rayleigh_in_scattering =
                    rayleigh_scattering * rayleigh_phase_value;

                let mie_in_scattering = mie_scattering * mie_phase_value;

                let in_scattering = (rayleigh_in_scattering
                    + mie_in_scattering)
                    * sun_transmittance;

                let scattering_integral = (in_scattering
                    - in_scattering * sample_transmittance)
                    / extinction;

                lum += scattering_integral * transmittance;
                transmittance *= sample_transmittance;
                step_i += 1.0;
            }

            if ground_distance > 0.0 {
                let mut hit_pos = pos + ground_distance * ray_dir;

                if pos.dot(sun_dir) > 0.0 {
                    hit_pos =
                        hit_pos.normalize() * Atmosphere::GROUND_RADIUS_MM;

                    lum += transmittance
                        * Atmosphere::GROUND_ALBEDO
                        * Atmosphere::sample_lut(
                            transmittance_lut_tex,
                            transmittance_lut_sampler,
                            hit_pos,
                            sun_dir,
                        );
                }
            }

            fms += lum_factor * inv_samples;
            lum_total += lum * inv_samples;
            j += 1;
        }

        i += 1;
    }

    (lum_total, fms)
}

#[spirv(compute(threads(8, 8)))]
#[allow(clippy::too_many_arguments)]
pub fn main_generate_sky_lut(
    #[spirv(global_invocation_id)] global_id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0, uniform)] world: &World,
    #[spirv(descriptor_set = 0, binding = 1)] transmittance_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 2)]
    transmittance_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 3)] scattering_lut_tex: Tex,
    #[spirv(descriptor_set = 0, binding = 4)] scattering_lut_sampler: &Sampler,
    #[spirv(descriptor_set = 0, binding = 5)] out: TexRgba16f,
) {
    let global_id = global_id.xy();
    let uv = global_id.as_vec2() / Atmosphere::SKY_LUT_RESOLUTION.as_vec2();

    let azimuth_angle = (uv.x - 0.5) * 2.0 * PI;

    let v = if uv.y < 0.5 {
        let coord = 1.0 - 2.0 * uv.y;

        -coord * coord
    } else {
        let coord = uv.y * 2.0 - 1.0;

        coord * coord
    };

    let height = Atmosphere::VIEW_POS.length();

    let horizon_angle = {
        let t = height * height
            - Atmosphere::GROUND_RADIUS_MM * Atmosphere::GROUND_RADIUS_MM;

        let t = t / height;

        t.sqrt().clamp(-1.0, 1.0).acos() - 0.5 * PI
    };

    let altitude_angle = v * 0.5 * PI - horizon_angle;

    let cos_altitude = altitude_angle.cos();

    let ray_dir = vec3(
        cos_altitude * azimuth_angle.sin(),
        altitude_angle.sin(),
        -cos_altitude * azimuth_angle.cos(),
    );

    let sun_dir = world.sun_direction();

    let atmosphere_distance = Ray::new(Atmosphere::VIEW_POS, ray_dir)
        .intersect_sphere(Atmosphere::ATMOSPHERE_RADIUS_MM);

    let ground_distance = Ray::new(Atmosphere::VIEW_POS, ray_dir)
        .intersect_sphere(Atmosphere::GROUND_RADIUS_MM);

    let t_max = if ground_distance < 0.0 {
        atmosphere_distance
    } else {
        ground_distance
    };

    let out_val = eval_sky(
        transmittance_lut_tex,
        transmittance_lut_sampler,
        scattering_lut_tex,
        scattering_lut_sampler,
        Atmosphere::VIEW_POS,
        ray_dir,
        sun_dir,
        t_max,
        Atmosphere::SKY_LUT_STEPS,
    );

    unsafe {
        out.write(global_id, out_val.extend(1.0));
    }
}

#[allow(clippy::too_many_arguments)]
fn eval_sky(
    transmittance_lut_tex: Tex,
    transmittance_lut_sampler: &Sampler,
    scattering_lut_tex: Tex,
    scattering_lut_sampler: &Sampler,
    pos: Vec3,
    ray_dir: Vec3,
    sun_dir: Vec3,
    t_max: f32,
    num_steps: f32,
) -> Vec3 {
    let cos_theta = ray_dir.dot(sun_dir);
    let mie_phase_value = get_mie_phase(cos_theta);
    let rayleigh_phase_value = get_rayleigh_phase(-cos_theta);

    let mut lum = Vec3::default();
    let mut transmittance = Vec3::splat(1.0);
    let mut t = 0.0;
    let mut i = 0.0;

    while i < num_steps {
        let new_t = ((i + 0.3) / num_steps) * t_max;
        let dt = new_t - t;

        t = new_t;

        let new_pos = pos + t * ray_dir;

        let (rayleigh_scattering, mie_scattering, extinction) =
            get_scattering_values(new_pos);

        let sample_transmittance = (-dt * extinction).exp();

        let sun_transmittance = Atmosphere::sample_lut(
            transmittance_lut_tex,
            transmittance_lut_sampler,
            new_pos,
            sun_dir,
        );

        let psi_ms = Atmosphere::sample_lut(
            scattering_lut_tex,
            scattering_lut_sampler,
            new_pos,
            sun_dir,
        );

        let rayleigh_in_scattering = rayleigh_scattering
            * (rayleigh_phase_value * sun_transmittance + psi_ms);

        let mie_in_scattering =
            mie_scattering * (mie_phase_value * sun_transmittance + psi_ms);

        let in_scattering = rayleigh_in_scattering + mie_in_scattering;

        let scattering_integral =
            (in_scattering - in_scattering * sample_transmittance) / extinction;

        lum += scattering_integral * transmittance;
        transmittance *= sample_transmittance;
        i += 1.0;
    }

    lum
}

fn get_scattering_values(pos: Vec3) -> (Vec3, f32, Vec3) {
    let altitude_km = (pos.length() - Atmosphere::GROUND_RADIUS_MM) * 1000.0;
    let rayleigh_density = (-altitude_km / 8.0).exp();
    let mie_density = (-altitude_km / 1.2).exp();

    let rayleigh_scattering =
        Atmosphere::RAYLEIGH_SCATTERING_BASE * rayleigh_density;

    let rayleigh_absorption =
        Atmosphere::RAYLEIGH_ABSORPTION_BASE * rayleigh_density;

    let mie_scattering = Atmosphere::MIE_SCATTERING_BASE * mie_density;
    let mie_absorption = Atmosphere::MIE_ABSORPTION_BASE * mie_density;

    let ozone_absorption = Atmosphere::OZONE_ABSORPTION_BASE
        * (1.0 - (altitude_km - 25.0).abs() / 15.0).max(0.0);

    let extinction = rayleigh_scattering
        + rayleigh_absorption
        + mie_scattering
        + mie_absorption
        + ozone_absorption;

    (rayleigh_scattering, mie_scattering, extinction)
}

fn get_mie_phase(cos_theta: f32) -> f32 {
    const G: f32 = 0.8;
    const SCALE: f32 = 3.0 / (8.0 * PI);

    let num = (1.0 - G * G) * (1.0 + cos_theta * cos_theta);
    let denom = (2.0 + G * G) * (1.0 + G * G - 2.0 * G * cos_theta).powf(1.5);

    SCALE * num / denom
}

fn get_rayleigh_phase(cos_theta: f32) -> f32 {
    const K: f32 = 3.0 / (16.0 * PI);

    K * (1.0 + cos_theta * cos_theta)
}

fn get_spherical_dir(theta: f32, phi: f32) -> Vec3 {
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    vec3(sin_phi * sin_theta, cos_phi, sin_phi * cos_theta)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
