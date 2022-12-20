mod builder;

use std::time::Instant;

use spirv_std::glam::Vec4;

use crate::{GeometryTris, StorageBufferable};

#[derive(Clone, Debug, Default)]
pub struct GeometryBvh {
    data: Vec<Vec4>,
}

impl GeometryBvh {
    pub fn rebuild(&mut self, scene: &GeometryTris) {
        log::trace!("Rebuilding ({} triangles)", scene.len());

        let tt = Instant::now();

        self.data.clear();

        let root = if scene.is_empty() {
            None
        } else {
            let root = builder::LinearBvh::build(scene);

            root.validate();

            Some(root)
        };

        builder::BvhSerializer::new(root.as_ref())
            .serialize_into(&mut self.data);

        log::trace!("Rebuilding completed (in {:?})", tt.elapsed());
    }
}

impl StorageBufferable for GeometryBvh {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
