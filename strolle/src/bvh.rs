mod axis;
mod bounding_box;
mod build;
mod bvh_node;
mod bvh_triangle;
mod serialize;

use std::fmt::Debug;

use spirv_std::glam::Vec4;

pub use self::axis::*;
pub use self::bounding_box::*;
pub use self::bvh_node::*;
pub use self::bvh_triangle::*;
use crate::{
    Bindable, BufferFlushOutcome, Instances, MappedStorageBuffer, Materials,
    Params, Triangles,
};

#[derive(Debug)]
pub struct Bvh {
    root: Option<BvhNode>,
    buffer: MappedStorageBuffer<Vec<Vec4>>,
}

impl Bvh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            root: None,
            buffer: MappedStorageBuffer::new_default(device, "bvh"),
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
        let triangle_count: usize = instances
            .iter()
            .filter_map(|(instance_handle, _)| triangles.count(instance_handle))
            .sum();

        if triangle_count == 0 {
            return;
        }

        let bvh_triangles =
            instances.iter().flat_map(|(inst_handle, instance)| {
                let material_id = materials.lookup(instance.material_handle());

                material_id.into_iter().flat_map(|material_id| {
                    triangles.iter(inst_handle).map(
                        move |(triangle_id, triangle)| BvhTriangle {
                            bb: BoundingBox::from_points(triangle.positions()),
                            center: triangle.center(),
                            triangle_id,
                            material_id,
                        },
                    )
                })
            });

        let root = build::run(self.root.as_ref(), bvh_triangles);

        self.buffer.clear();
        self.buffer.reserve_exact((2 * triangle_count - 1) * 2);

        serialize::run(&mut self.buffer, &root);

        self.root = Some(root);
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        self.buffer.flush(device, queue)
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}
