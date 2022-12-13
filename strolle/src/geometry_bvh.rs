mod builder;

use spirv_std::glam::Vec4;

use crate::{GeometryTris, StorageBufferable};

#[derive(Clone, Debug, Default)]
pub struct GeometryBvh {
    data: Vec<Vec4>,
}

impl GeometryBvh {
    pub fn rebuild(&mut self, tris: &GeometryTris) {
        let bvh = builder::Bvh::new(tris);
        let rbvh = builder::RopedBvh::new(bvh);

        builder::serialize(&mut self.data, rbvh);
    }
}

impl StorageBufferable for GeometryBvh {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
