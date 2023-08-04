pub trait U32Ext
where
    Self: Sized,
{
    fn from_bytes(bytes: [u32; 4]) -> Self;
    fn to_bytes(self) -> [u32; 4];
}

impl U32Ext for u32 {
    fn from_bytes([a, b, c, d]: [u32; 4]) -> Self {
        a | (b << 8) | (c << 16) | (d << 24)
    }

    fn to_bytes(mut self) -> [u32; 4] {
        let a = self & 0xff;
        self >>= 8;
        let b = self & 0xff;
        self >>= 8;
        let c = self & 0xff;
        self >>= 8;
        let d = self & 0xff;

        [a, b, c, d]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_to_bytes() {
        assert_eq!(0xcafebabe, u32::from_bytes(u32::to_bytes(0xcafebabe)));
    }
}
