use std::ops::BitXor;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MortonCode(pub(super) u64);

impl BitXor for MortonCode {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl MortonCode {
    pub fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }
}
