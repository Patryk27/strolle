use strolle_gpu::prelude::*;

use crate::utils::{
    eval_mie_phase, eval_rayleigh_phase, eval_scattering, spherical_direction,
};

pub fn main(
    global_id: UVec3,
    transmittance_lut_tex: Tex,
    transmittance_lut_sampler: &Sampler,
    out: TexRgba16f,
) {
    let global_id = global_id.xy();

    let uv =
        global_id.as_vec2() / Atmosphere::SCATTERING_LUT_RESOLUTION.as_vec2();

    let sun_cos_theta = 2.0 * uv.x - 1.0;
    let sun_theta = sun_cos_theta.clamp(-1.0, 1.0).acos();

    let height = lerp(
        Atmosphere::GROUND_RADIUS_MM,
        Atmosphere::ATMOSPHERE_RADIUS_MM,
        uv.y.max(0.01),
    );

    let pos = vec3(0.0, height, 0.0);
    let sun_dir = vec3(0.0, sun_cos_theta, -sun_theta.sin()).normalize();

    let (lum, f_ms) = eval(
        transmittance_lut_tex,
        transmittance_lut_sampler,
        pos,
        sun_dir,
    );

    let out_val = lum / (1.0 - f_ms);

    unsafe {
        out.write(global_id, out_val.extend(1.0));
    }
}

pub fn eval(
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

            let ray_dir = spherical_direction(theta, phi);

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
            let mie_phase_value = eval_mie_phase(cos_theta);
            let rayleigh_phase_value = eval_rayleigh_phase(-cos_theta);

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
                    eval_scattering(new_pos);

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
