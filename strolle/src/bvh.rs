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
    buffer: MappedStorageBuffer<Vec<Vec4>>,
}

impl Bvh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
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
        // TODO it would be nice not to re-collect all triangles every frame
        //      (it doesn't seem to be a bottleneck, though)
        let mut triangles: Vec<_> = instances
            .iter()
            .flat_map(|(instance_handle, instance_entry)| {
                let material_id =
                    materials.lookup(&instance_entry.instance.material_handle);

                material_id.into_iter().flat_map(|material_id| {
                    triangles.iter(instance_handle).map(
                        move |(triangle_id, triangle)| BvhTriangle {
                            bounds: triangle.positions().into_iter().collect(),
                            center: triangle.center(),
                            triangle_id,
                            material_id,
                        },
                    )
                })
            })
            .collect();

        if triangles.is_empty() {
            return;
        }

        // TODO it would be nice not to re-allocate the nodes every frame
        //      (it doesn't seem to be a bottleneck, though)
        let mut nodes = Vec::new();

        // BVH, being a binary tree, can consist of at most 2 * leafes - 1
        // nodes, so let's preallocate it:
        nodes.resize(2 * triangles.len() - 1, Default::default());

        let nodes = build::run(nodes, &mut triangles);

        self.buffer.clear();

        serialize::run(materials, &nodes, &mut self.buffer);
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
