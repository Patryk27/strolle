use strolle_gpu::prelude::*;

use super::utils::*;

pub fn main(
    global_id: UVec3,
    world: &World,
    transmittance_lut_tex: Tex,
    transmittance_lut_sampler: &Sampler,
    scattering_lut_tex: Tex,
    scattering_lut_sampler: &Sampler,
    out: TexRgba16,
) {
    let global_id = global_id.xy();
    let uv = global_id.as_vec2() / Atmosphere::SKY_LUT_RESOLUTION.as_vec2();

    let ray_dir = {
        let azimuth = (uv.x - 0.5) * 2.0 * PI;

        let altitude = {
            let v = if uv.y < 0.5 {
                let coord = 1.0 - 2.0 * uv.y;

                -coord * coord
            } else {
                let coord = uv.y * 2.0 - 1.0;

                coord * coord
            };

            let horizon = {
                let height = Atmosphere::VIEW_POS.length();
                let t = height.sqr() - Atmosphere::GROUND_RADIUS_MM.sqr();
                let t = t.sqrt() / height;

                t.clamp(-1.0, 1.0).acos() - 0.5 * PI
            };

            v * 0.5 * PI - horizon
        };

        vec3(
            altitude.cos() * azimuth.sin(),
            altitude.sin(),
            -altitude.cos() * azimuth.cos(),
        )
    };

    let sun_dir = {
        let altitude = world.sun_altitude % (2.0 * PI);

        if altitude < 0.5 * PI {
            vec3(0.0, altitude.sin(), -altitude.cos())
        } else {
            // There's (probably) something wrong with the way we compute
            // azimuth during sky-lut sampling which shows up as sun being on
            // the opposite side of the sky when it's setting down.
            //
            // I'm not sure what's wrong with our sampling there, so let's
            // hotfix it here.
            vec3(0.0, altitude.sin(), altitude.cos())
        }
    };

    let atmosphere_distance = Ray::new(Atmosphere::VIEW_POS, ray_dir)
        .intersect_sphere(Atmosphere::ATMOSPHERE_RADIUS_MM);

    let ground_distance = Ray::new(Atmosphere::VIEW_POS, ray_dir)
        .intersect_sphere(Atmosphere::GROUND_RADIUS_MM);

    let t_max = if ground_distance < 0.0 {
        atmosphere_distance
    } else {
        ground_distance
    };

    let out_val = eval(
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
pub fn eval(
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
    let mie_phase_value = eval_mie_phase(cos_theta);
    let rayleigh_phase_value = eval_rayleigh_phase(-cos_theta);

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
            eval_scattering(new_pos);

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
