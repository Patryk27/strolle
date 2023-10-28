mod builder;
mod node;
mod nodes;
mod primitive;
mod primitives;
mod serializer;

use std::fmt::Debug;
use std::ops::Range;

use spirv_std::glam::Vec4;

pub use self::builder::*;
pub use self::node::*;
pub use self::nodes::*;
pub use self::primitive::*;
pub use self::primitives::*;
use crate::{
    utils, Bindable, BufferFlushOutcome, MappedStorageBuffer, Materials, Params,
};

#[derive(Debug)]
pub struct Bvh {
    buffer: MappedStorageBuffer<Vec<Vec4>>,
    nodes: BvhNodes,
    primitives: BvhPrimitives,
}

impl Bvh {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: MappedStorageBuffer::new_default(device, "bvh"),
            nodes: Default::default(),
            primitives: Default::default(),
        }
    }

    pub fn add(&mut self, prim: BvhPrimitive) {
        self.primitives.add(prim);
    }

    pub fn update(
        &mut self,
        ids: Range<usize>,
    ) -> impl Iterator<Item = &mut BvhPrimitive> {
        self.primitives.update(ids)
    }

    pub fn refresh<P>(&mut self, materials: &Materials<P>)
    where
        P: Params,
    {
        utils::measure("flush.bvh.refresh.begin", || {
            self.primitives.begin_refresh();
        });

        utils::measure("flush.bvh.refresh.builder", || {
            builder::run(&mut self.nodes, &mut self.primitives);
        });

        utils::measure("flush.bvh.refresh.serializer", || {
            serializer::run(
                materials,
                &self.nodes,
                &self.primitives,
                &mut self.buffer,
            );
        });

        self.primitives.end_refresh();
    }

    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> BufferFlushOutcome {
        self.buffer.flush(device, queue)
    }

    pub fn len(&self) -> usize {
        self.nodes.nodes.len()
    }

    pub fn bind_readable(&self) -> impl Bindable + '_ {
        self.buffer.bind_readable()
    }
}
