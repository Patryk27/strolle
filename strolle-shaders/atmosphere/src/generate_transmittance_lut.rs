use strolle_gpu::prelude::*;

use crate::utils::eval_scattering;

pub fn main(global_id: UVec3, out: TexRgba16f) {
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
    let out_val = eval(pos, sun_dir);

    unsafe {
        out.write(global_id, out_val.extend(1.0));
    }
}

fn eval(pos: Vec3, sun_dir: Vec3) -> Vec3 {
    if Ray::new(pos, sun_dir).intersect_sphere(Atmosphere::GROUND_RADIUS_MM)
        > 0.0
    {
        return Default::default();
    }

    let atmosphere_distance = Ray::new(pos, sun_dir)
        .intersect_sphere(Atmosphere::ATMOSPHERE_RADIUS_MM);

    let mut t = 0.0;
    let mut transmittance = Vec3::splat(1.0);
    let mut i = 0.0;

    while i < Atmosphere::TRANSMITTANCE_LUT_STEPS {
        let new_t = ((i + 0.3) / Atmosphere::TRANSMITTANCE_LUT_STEPS)
            * atmosphere_distance;

        let dt = new_t - t;

        t = new_t;

        let new_pos = pos + t * sun_dir;
        let (_, _, extinction) = eval_scattering(new_pos);

        transmittance *= (-dt * extinction).exp();
        i += 1.0;
    }

    transmittance
}
