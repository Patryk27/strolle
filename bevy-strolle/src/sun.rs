use std::ops::{Deref, DerefMut};

use bevy::prelude::Resource;
use strolle as st;

#[derive(Clone, Debug, Default, Resource)]
pub struct StrolleSun {
    sun: st::Sun,
}

impl Deref for StrolleSun {
    type Target = st::Sun;

    fn deref(&self) -> &Self::Target {
        &self.sun
    }
}

impl DerefMut for StrolleSun {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sun
    }
}
