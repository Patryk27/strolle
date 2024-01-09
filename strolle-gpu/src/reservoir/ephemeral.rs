use core::ops::{Deref, DerefMut};

use glam::Vec3;

use crate::{LightId, Reservoir, Vec3Ext};

#[derive(Clone, Copy, Default)]
pub struct EphemeralReservoir {
    pub reservoir: Reservoir<EphemeralSample>,
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
    pub light_radiance: Vec3,
}

impl EphemeralSample {
    pub fn pdf(&self) -> f32 {
        self.light_radiance.luminance()
    }
}
