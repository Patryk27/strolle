use strolle_gpu::prelude::*;

pub fn eval_scattering(pos: Vec3) -> (Vec3, f32, Vec3) {
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

pub fn eval_mie_phase(cos_theta: f32) -> f32 {
    const G: f32 = 0.8;
    const SCALE: f32 = 3.0 / (8.0 * PI);

    let num = (1.0 - G * G) * (1.0 + cos_theta * cos_theta);
    let denom = (2.0 + G * G) * (1.0 + G * G - 2.0 * G * cos_theta).powf(1.5);

    SCALE * num / denom
}

pub fn eval_rayleigh_phase(cos_theta: f32) -> f32 {
    const K: f32 = 3.0 / (16.0 * PI);

    K * (1.0 + cos_theta * cos_theta)
}

pub fn spherical_direction(theta: f32, phi: f32) -> Vec3 {
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    vec3(sin_phi * sin_theta, cos_phi, sin_phi * cos_theta)
}
