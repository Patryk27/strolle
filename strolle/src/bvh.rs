mod bounding_box;
mod builders;
mod bvh_node;
mod bvh_printer;
mod bvh_serializer;
mod bvh_triangle;

use std::fmt::Debug;

use spirv_std::glam::Vec4;

pub use self::bounding_box::*;
pub use self::bvh_node::*;
pub use self::bvh_serializer::*;
pub use self::bvh_triangle::*;
use crate::{
    Bindable, Instances, MappedStorageBuffer, Materials, Params, Triangles,
};

const ALGORITHM: &str = "lbvh";

#[derive(Debug)]
pub struct Bvh {
    buffer: MappedStorageBuffer<Vec<Vec4>>,
}

impl Bvh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: MappedStorageBuffer::new_default(
                device,
                "strolle_bvh",
                32 * 1024 * 1024,
            ),
        }
    }

    pub fn refresh<P>(
        &mut self,
        instances: &Instances<P>,
        materials: &Materials<P>,
        triangles: &Triangles<P>,
    ) where
        P: Params,
    {
        // TODO rebuilding BVHs for all meshes here might be pretty intensive;
        //      consider using BLAS + TLAS (at least for internal purposes)
        let bvh_triangles: Vec<_> = instances
            .iter()
            .flat_map(|(instance_handle, instance)| {
                let material = materials.lookup(instance.material_handle());

                material.into_iter().flat_map(|material_id| {
                    triangles.iter(instance_handle).map(
                        move |(triangle_id, triangle)| BvhTriangle {
                            triangle,
                            triangle_id,
                            material_id,
                        },
                    )
                })
            })
            .collect();

        if bvh_triangles.is_empty() {
            return;
        }

        let root = match ALGORITHM {
            "lbvh" => builders::lbvh::build(bvh_triangles),
            "sah" => builders::sah::build(bvh_triangles),
            _ => unreachable!(),
        };

        root.validate();

        self.buffer.clear();

        BvhSerializer::process(&mut self.buffer, &root);
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        self.buffer.flush(queue);
    }

    pub fn as_ro_bind(&self) -> impl Bindable + '_ {
        self.buffer.as_ro_bind()
    }
}
