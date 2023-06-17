#[derive(Clone, Debug)]
pub struct Sun {
    pub altitude: f32,
}

impl Default for Sun {
    fn default() -> Self {
        Self { altitude: 0.35 }
    }
}
