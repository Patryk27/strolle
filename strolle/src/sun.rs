#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sun {
    pub azimuth: f32,
    pub altitude: f32,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            azimuth: 0.0,
            altitude: 0.35,
        }
    }
}
