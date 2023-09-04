use core::ops::{Deref, DerefMut};

use glam::{Vec3, Vec4Swizzles};

use crate::{
    Atmosphere, Hit, LightId, LightsView, Reservoir, Vec3Ext, WhiteNoise, World,
};

/// Reservoir for sampling lights temporarily, without storing them in-between
/// frames.
///
/// See: [`Reservoir`].
#[derive(Clone, Copy, Default)]
pub struct EphemeralReservoir {
    reservoir: Reservoir<EphemeralReservoirSample>,
}

impl EphemeralReservoir {
    /// Samples scene's lightning, including the sun, and returns chosen light's
    /// id, probability and radiance.
    pub fn sample<const INDIRECT: bool>(
        wnoise: &mut WhiteNoise,
        atmosphere: &Atmosphere,
        world: &World,
        lights: &LightsView,
        hit: Hit,
    ) -> (LightId, f32, Vec3) {
        let light_id;
        let light_pdf;
        let light_radiance;

        if hit.is_none() {
            light_id = LightId::sun();
            light_pdf = 1.0;

            light_radiance =
                atmosphere.sky(world.sun_direction(), hit.direction);
        } else {
            let mut reservoir = Self::default();
            let mut light_idx = 0;

            while light_idx < world.light_count {
                let light_id = LightId::new(light_idx);

                let light_radiance = if INDIRECT {
                    lights.get(light_id).contribution(hit)
                } else {
                    lights.get(light_id).radiance(hit)
                };

                let sample = EphemeralReservoirSample {
                    light_id,
                    light_radiance,
                };

                reservoir.add(wnoise, sample, sample.p_hat());
                light_idx += 1;
            }

            // ---

            let sample = {
                let albedo = if INDIRECT {
                    hit.gbuffer.base_color.xyz() * (1.0 - hit.gbuffer.metallic)
                } else {
                    Vec3::ONE
                };

                let cosine =
                    hit.gbuffer.normal.dot(world.sun_direction()).max(0.0);

                EphemeralReservoirSample {
                    light_id: LightId::sun(),
                    light_radiance: atmosphere.sun(world.sun_direction())
                        * albedo
                        * cosine,
                }
            };

            reservoir.add(wnoise, sample, sample.p_hat());

            // ---

            if reservoir.w_sum > 0.0 {
                light_id = reservoir.sample.light_id;
                light_pdf = reservoir.sample.p_hat() / reservoir.w_sum;
                light_radiance = reservoir.sample.light_radiance;
            } else {
                light_id = LightId::new(0);
                light_pdf = 0.0;
                light_radiance = Vec3::ZERO;
            }
        }

        (light_id, light_pdf, light_radiance)
    }
}

impl Deref for EphemeralReservoir {
    type Target = Reservoir<EphemeralReservoirSample>;

    fn deref(&self) -> &Self::Target {
        &self.reservoir
    }
}

impl DerefMut for EphemeralReservoir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reservoir
    }
}

#[derive(Clone, Copy, Default)]
pub struct EphemeralReservoirSample {
    pub light_id: LightId,
    pub light_radiance: Vec3,
}

impl EphemeralReservoirSample {
    pub fn p_hat(&self) -> f32 {
        self.light_radiance.luminance()
    }
}
