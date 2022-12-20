use core::ops;

use crate::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct PadU32 {
    pub value: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

impl PadU32 {
    pub const fn new(value: u32) -> Self {
        Self {
            value,
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        }
    }
}

impl From<u32> for PadU32 {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<PadU32> for u32 {
    fn from(pad: PadU32) -> Self {
        pad.value
    }
}

impl ops::Add<u32> for PadU32 {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self::new(self.value + rhs)
    }
}

impl ops::AddAssign<u32> for PadU32 {
    fn add_assign(&mut self, rhs: u32) {
        self.value += rhs;
    }
}

impl ops::Sub<u32> for PadU32 {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self::new(self.value - rhs)
    }
}

impl ops::SubAssign<u32> for PadU32 {
    fn sub_assign(&mut self, rhs: u32) {
        self.value -= rhs;
    }
}
