mod builder;
mod node;
mod nodes;
mod serializer;

use std::fmt::Debug;

use spirv_std::glam::Vec4;

pub use self::builder::*;
pub use self::node::*;
pub use self::nodes::*;
use crate::instances::Instances;
use crate::primitives::BlasPrimitives;
use crate::{
    utils, Bindable, BufferFlushOutcome, MappedStorageBuffer, Materials,
    Params, Primitives,
};

#[derive(Debug)]
pub struct Bvh {
    buffer: MappedStorageBuffer<Vec<Vec4>>,
    nodes: BvhNodes,
}

impl Bvh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: MappedStorageBuffer::new_default(device, "bvh"),
            nodes: Default::default(),
        }
    }

    pub fn create_blas(
        &mut self,
        primitives: &mut BlasPrimitives,
    ) -> BvhNodeId {
        builder::run_blas(primitives, &mut self.nodes)
    }

    pub fn delete_blas(&mut self, node_id: BvhNodeId) {
        self.nodes.remove_tree(node_id);
    }

    pub fn refresh<P>(
        &mut self,
        materials: &Materials<P>,
        instances: &Instances<P>,
        primitives: &mut Primitives<P>,
    ) where
        P: Params,
    {
        utils::measure("tick.bvh.begin", || {
            primitives.tlas_mut().begin();
        });

        utils::measure("tick.bvh.build", || {
            builder::run_tlas(primitives.tlas_mut(), &mut self.nodes);
        });

        utils::measure("tick.bvh.serialize", || {
            serializer::run(
                materials,
                instances,
                primitives,
                &self.nodes,
                &mut self.buffer,
            );
        });

        utils::measure("tick.bvh.end", || {
            primitives.tlas_mut().commit();
        });
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        self.buffer.flush(device, queue)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}
