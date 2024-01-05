mod builder;
mod node;
mod nodes;
mod primitive;
mod primitives;
mod serializer;

use std::ops::Range;

use bevy::ecs::system::Resource;
use bevy::ecs::world::FromWorld;
use bevy::prelude::World;
use bevy::render::render_resource::{BufferVec, IntoBinding};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use spirv_std::glam::Vec4;
use wgpu::BufferUsages;

pub use self::builder::*;
pub use self::node::*;
pub use self::nodes::*;
pub use self::primitive::*;
pub use self::primitives::*;
use crate::{utils, Materials};

#[derive(Resource)]
pub struct Bvh {
    buffer: BufferVec<Vec4>,
    nodes: BvhNodes,
    primitives: BvhPrimitives,
}

impl Bvh {
    pub fn add(&mut self, prim: BvhPrimitive) {
        self.primitives.add(prim);
    }

    pub fn update(
        &mut self,
        ids: Range<usize>,
    ) -> impl Iterator<Item = &mut BvhPrimitive> {
        self.primitives.update(ids)
    }

    pub fn len(&self) -> usize {
        self.nodes.nodes.len()
    }

    pub fn refresh(&mut self, materials: &Materials) {
        utils::measure("refresh.bvh.begin", || {
            self.primitives.begin_refresh();
        });

        utils::measure("refresh.bvh.build", || {
            builder::run(&mut self.nodes, &mut self.primitives);
        });

        utils::measure("refresh.bvh.serialize", || {
            serializer::run(
                materials,
                &self.nodes,
                &self.primitives,
                self.buffer.values_mut(),
            );
        });

        self.primitives.end_refresh();
    }

    pub fn flush(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.buffer.write_buffer(device, queue);
    }

    pub fn bind(&self) -> impl IntoBinding {
        self.buffer
            .buffer()
            .expect("buffer not ready: bvh")
            .as_entire_buffer_binding()
    }
}

impl FromWorld for Bvh {
    fn from_world(_: &mut World) -> Self {
        Self {
            buffer: BufferVec::new(BufferUsages::STORAGE),
            nodes: Default::default(),
            primitives: Default::default(),
        }
    }
}
