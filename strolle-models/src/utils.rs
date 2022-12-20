mod padu32;

pub use self::padu32::*;

#[derive(Clone, Copy)]
pub enum Culling {
    Enabled,
    Disabled,
}

impl Culling {
    pub fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}
