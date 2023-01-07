use std::fmt::Debug;

use spirv_std::glam::Vec3;
use strolle_models as gpu;

use crate::bvh::{builders, BoundingBox, BvhNode, BvhNodePayload, BvhObject};
use crate::Instances;

// TODO
const ALGO: &str = "lbvh";

#[derive(Clone, Debug)]
pub struct WorldBvh {
    root: BvhNode,
}

impl WorldBvh {
    pub fn build(instances: &Instances) -> Self {
        let objects: Vec<_> = instances
            .iter()
            .map(|(id, _, bounding_box)| Object { id, bounding_box })
            .collect();

        let root = match ALGO {
            "lbvh" => builders::lbvh::build(&objects),
            "sah" => builders::sah::build(&objects),
            _ => unreachable!(),
        };

        root.validate();

        Self { root }
    }

    pub fn root(&self) -> &BvhNode {
        &self.root
    }
}

#[derive(Clone, Debug)]
struct Object {
    id: gpu::InstanceId,
    bounding_box: BoundingBox,
}

impl BvhObject for Object {
    fn payload(&self) -> BvhNodePayload {
        BvhNodePayload::Instance(self.id)
    }

    fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    fn center(&self) -> Vec3 {
        self.bounding_box.center()
    }
}
