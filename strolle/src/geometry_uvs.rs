use spirv_std::glam::Vec4;

use crate::StorageBufferable;

#[derive(Clone, Debug, Default)]
pub struct GeometryUvs {
    data: Vec<Vec4>,
}

impl StorageBufferable for GeometryUvs {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
