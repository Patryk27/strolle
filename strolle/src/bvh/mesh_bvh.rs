use spirv_std::glam::Vec3;
use strolle_models::{MeshTriangleId, Triangle};

use crate::bvh::{builders, BoundingBox, BvhNode, BvhNodePayload, BvhObject};

#[derive(Clone, Debug)]
pub struct MeshBvh {
    root: BvhNode,
}

impl MeshBvh {
    pub fn build(triangles: &[Triangle]) -> Self {
        let objects: Vec<_> = triangles
            .iter()
            .enumerate()
            .map(|(id, triangle)| Object {
                id: MeshTriangleId::new(id as u32),
                triangle,
            })
            .collect();

        let root = builders::lbvh::build(&objects);

        root.validate();

        Self { root }
    }

    pub fn root(&self) -> &BvhNode {
        &self.root
    }
}

#[derive(Clone, Debug)]
struct Object<'a> {
    id: MeshTriangleId,
    triangle: &'a Triangle,
}

impl BvhObject for Object<'_> {
    fn payload(&self) -> BvhNodePayload {
        BvhNodePayload::Triangle(self.id)
    }

    fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_points(self.triangle.vertices())
    }

    fn center(&self) -> Vec3 {
        self.triangle.vertices().into_iter().sum::<Vec3>() / 3.0
    }
}
