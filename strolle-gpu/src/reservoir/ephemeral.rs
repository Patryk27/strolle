use core::ops::{Deref, DerefMut};

use crate::{
    Hit, LightId, LightRadiance, LightsView, Reservoir, Vec3Ext, WhiteNoise,
    World,
};

#[derive(Clone, Copy, Default)]
pub struct EphemeralReservoir {
    pub reservoir: Reservoir<EphemeralSample>,
}

impl EphemeralReservoir {
    pub fn build(
        wnoise: &mut WhiteNoise,
        lights: LightsView,
        world: World,
        hit: Hit,
    ) -> Self {
        let mut res = EphemeralReservoir::default();
        let mut res_pdf = 0.0;

        // TODO rust-gpu seems to miscompile `.min()`
        let max_samples = if world.light_count < 16 {
            world.light_count
        } else {
            16
        };

        let sample_ipdf = world.light_count as f32;
        let mut sample_nth = 0;

        while sample_nth < max_samples {
            let light_id =
                LightId::new(wnoise.sample_int() % world.light_count);

            let light_rad = lights.get(light_id).radiance(hit);

            let sample = EphemeralSample {
                light_id,
                light_rad,
            };

            let sample_pdf = sample.pdf();

            if res.update(wnoise, sample, sample_pdf * sample_ipdf) {
                res_pdf = sample_pdf;
            }

            sample_nth += 1;
        }

        res.norm_avg(res_pdf);
        res
    }
}

impl Deref for EphemeralReservoir {
    type Target = Reservoir<EphemeralSample>;

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
pub struct EphemeralSample {
    pub light_id: LightId,
    pub light_rad: LightRadiance,
}

impl EphemeralSample {
    pub fn pdf(self) -> f32 {
        self.light_rad.radiance.perc_luma()
    }
}
