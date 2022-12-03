use crate::*;

#[derive(Copy, Clone, Default)]
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct TriangleUv {
    pub uv0: Vec2,
    pub uv1: Vec2,
    pub uv2: Vec2,
}

impl TriangleUv {
    pub fn new(uv0: Vec2, uv1: Vec2, uv2: Vec2) -> Self {
        Self { uv0, uv1, uv2 }
    }
}
