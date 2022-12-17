mod builder;

use spirv_std::glam::Vec4;

use crate::{GeometryTris, StorageBufferable};

#[derive(Clone, Debug, Default)]
pub struct GeometryBvh {
    data: Vec<Vec4>,
}

impl GeometryBvh {
    pub fn rebuild(&mut self, scene: &GeometryTris) {
        self.data.clear();

        if scene.is_empty() {
            return;
        }

        // {
        //     let root = builder::SahBvh::build(scene);
        //     let rbvh = builder::RopedBvh::build(&root);

        //     std::fs::write("/tmp/bvh.old.dot", rbvh.to_string()).unwrap();

        //     rbvh.serialize_into(&mut self.data);
        // }

        {
            let root = builder::LinearBvh::build(scene);
            let rbvh = builder::RopedBvh::build(&root);

            // std::fs::write("/tmp/bvh.new.dot", rbvh.to_string()).unwrap();

            rbvh.serialize_into(&mut self.data);
        }
    }
}

impl StorageBufferable for GeometryBvh {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
