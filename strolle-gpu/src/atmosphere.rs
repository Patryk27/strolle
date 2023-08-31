use core::f32::consts::PI;

use glam::{uvec2, vec2, vec3, UVec2, Vec3, Vec4Swizzles};
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::Sampler;

use crate::{F32Ext, Ray, Tex};

pub struct Atmosphere<'a> {
    transmittance_lut_tex: Tex<'a>,
    transmittance_lut_sampler: &'a Sampler,
    sky_lut_tex: Tex<'a>,
    sky_lut_sampler: &'a Sampler,
}

impl<'a> Atmosphere<'a> {
    /// Resolution of the transmittance lookup texture.
    ///
    /// This texture is generated just once, when Strolle is booting.
    pub const TRANSMITTANCE_LUT_RESOLUTION: UVec2 = uvec2(256, 64);

    /// Quality of the transmittance lookup texture.
    pub const TRANSMITTANCE_LUT_STEPS: f32 = 40.0;

    /// Resolution of the scattering lookup texture.
    ///
    /// This texture is generated just once, when Strolle is booting.
    pub const SCATTERING_LUT_RESOLUTION: UVec2 = uvec2(32, 32);

    /// Quality of the scattering lookup texture.
    pub const SCATTERING_LUT_STEPS: f32 = 20.0;

    /// Quality of the scattering lookup texture.
    pub const SCATTERING_LUT_SAMPLES_SQRT: usize = 8;

    /// Resolution of the sky lookup texture.
    ///
    /// This texture is regenerated each time sun's position changes so it's
    /// important not to go too crazy in here.
    pub const SKY_LUT_RESOLUTION: UVec2 = uvec2(256, 256);

    /// Quality of the sky lookup texture.
    pub const SKY_LUT_STEPS: f32 = 32.0;

    /// Radius of the planet, in mega-meters.
    pub const GROUND_RADIUS_MM: f32 = 6.360;

    /// Radius of the atmosphere, in mega-meters;
    pub const ATMOSPHERE_RADIUS_MM: f32 = 6.460;

    pub const RAYLEIGH_SCATTERING_BASE: Vec3 = vec3(5.802, 13.558, 33.1);
    pub const RAYLEIGH_ABSORPTION_BASE: f32 = 0.0;

    pub const MIE_SCATTERING_BASE: f32 = 3.996;
    pub const MIE_ABSORPTION_BASE: f32 = 4.4;

    pub const OZONE_ABSORPTION_BASE: Vec3 = vec3(0.650, 1.881, 0.085);

    pub const GROUND_ALBEDO: Vec3 = Vec3::splat(0.25);

    pub const EXPOSURE: f32 = 20.0;

    /// Position of the observer in world.
    ///
    /// This is a constant because the atmosphere generally doesn't change that
    /// much when camera is moving (unless one's travelling in a spaceship) and
    /// so it's just more practical to use a hard-coded value here.
    pub const VIEW_POS: Vec3 = vec3(0.0, Self::GROUND_RADIUS_MM + 0.0002, 0.0);

    pub fn new(
        transmittance_lut_tex: Tex<'a>,
        transmittance_lut_sampler: &'a Sampler,
        sky_lut_tex: Tex<'a>,
        sky_lut_sampler: &'a Sampler,
    ) -> Self {
        Self {
            transmittance_lut_tex,
            transmittance_lut_sampler,
            sky_lut_tex,
            sky_lut_sampler,
        }
    }

    /// Returns color of the sun at given direction.
    pub fn sun(&self, sun_dir: Vec3) -> Vec3 {
        self.sample_transmittance_lut(Self::VIEW_POS, sun_dir) * Self::EXPOSURE
    }

    /// Returns color of the sky at given direction.
    pub fn sky(&self, sun_dir: Vec3, ray_dir: Vec3) -> Vec3 {
        let mut lum = self.sample_sky_lut(ray_dir, sun_dir);
        let mut sun_lum = self.evaluate_bloom(ray_dir, sun_dir);

        sun_lum = self.interpolate_bloom(sun_lum);

        if sun_lum.length_squared() > 0.0 {
            let ray = Ray::new(Self::VIEW_POS, ray_dir);

            if ray.intersect_sphere(Self::GROUND_RADIUS_MM) >= 0.0 {
                sun_lum = Vec3::ZERO;
            } else {
                sun_lum *=
                    self.sample_transmittance_lut(Self::VIEW_POS, sun_dir);
            }
        }

        lum += sun_lum;
        lum *= Self::EXPOSURE;
        lum
    }

    fn sample_sky_lut(&self, ray_dir: Vec3, sun_dir: Vec3) -> Vec3 {
        let height = Self::VIEW_POS.length();
        let up = Self::VIEW_POS / height;

        let horizon = {
            let t = height.sqr() - Self::GROUND_RADIUS_MM.sqr();
            let t = t.sqrt() / height;

            t.clamp(-1.0, 1.0).acos()
        };

        let altitude = horizon - ray_dir.dot(up).acos();

        let azimuth = if altitude.abs() > (0.5 * PI - 0.0001) {
            0.0
        } else {
            let right = sun_dir.cross(up);
            let forward = up.cross(right);

            let projected_dir = (ray_dir - up * ray_dir.dot(up)).normalize();
            let sin_theta = projected_dir.dot(right);
            let cos_theta = projected_dir.dot(forward);

            sin_theta.atan2(cos_theta) + PI
        };

        let uv = {
            let u = azimuth / (2.0 * PI);

            let v = 0.5
                + 0.5 * (altitude.abs() * 2.0 / PI).sqrt().copysign(altitude);

            vec2(u, v)
        };

        self.sky_lut_tex
            .sample_by_lod(*self.sky_lut_sampler, uv, 0.0)
            .xyz()
    }

    fn evaluate_bloom(&self, ray_dir: Vec3, sun_dir: Vec3) -> Vec3 {
        const SUN_SOLID_ANGLE: f32 = 0.53 * PI / 180.0;

        let min_sun_cos_theta = SUN_SOLID_ANGLE.cos();
        let cos_theta = ray_dir.dot(sun_dir);

        if cos_theta >= min_sun_cos_theta {
            return Vec3::splat(1.0);
        }

        let offset = min_sun_cos_theta - cos_theta;
        let gaussian_bloom = (-offset * 50000.0).exp() * 0.5;
        let inv_bloom = 1.0 / (0.02 + offset * 300.0) * 0.01;

        Vec3::splat(gaussian_bloom + inv_bloom)
    }

    fn interpolate_bloom(&self, bloom: Vec3) -> Vec3 {
        const MIN: Vec3 = Vec3::splat(0.002);
        const MAX: Vec3 = Vec3::splat(1.0);

        let t = ((bloom - MIN) / (MAX - MIN)).clamp(Vec3::ZERO, Vec3::ONE);

        t * t * (3.0 - 2.0 * t)
    }

    fn sample_transmittance_lut(&self, pos: Vec3, sun_dir: Vec3) -> Vec3 {
        Self::sample_lut(
            self.transmittance_lut_tex,
            self.transmittance_lut_sampler,
            pos,
            sun_dir,
        )
    }

    pub fn sample_lut(
        lut_tex: Tex,
        lut_sampler: &Sampler,
        pos: Vec3,
        sun_dir: Vec3,
    ) -> Vec3 {
        let height = pos.length();
        let up = pos / height;
        let sun_cos_zenith_angle = sun_dir.dot(up);

        let uv = {
            let u = (0.5 + 0.5 * sun_cos_zenith_angle).saturate();

            let v = ((height - Self::GROUND_RADIUS_MM)
                / (Self::ATMOSPHERE_RADIUS_MM - Self::GROUND_RADIUS_MM))
                .saturate();

            vec2(u, v)
        };

        lut_tex.sample_by_lod(*lut_sampler, uv, 0.0).xyz()
    }
}
