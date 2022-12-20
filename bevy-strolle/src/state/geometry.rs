use bevy::render::renderer::RenderQueue;

use crate::*;

#[derive(Default)]
pub struct Geometry {
    tris: st::GeometryTris,
    uvs: st::GeometryUvs,
    bvh: st::GeometryBvh,
    dirty: bool,
}

impl Geometry {
    pub fn alloc(&mut self, tri: st::Triangle) {
        self.tris.push(tri);
    }

    pub fn reindex(&mut self) {
        self.bvh.rebuild(&self.tris);
        self.dirty = true;
    }

    pub fn write(&mut self, engine: &st::Engine, queue: &RenderQueue) {
        if !self.dirty {
            return;
        }

        engine.write_geometry(
            queue.0.as_ref(),
            &self.tris,
            &self.uvs,
            &self.bvh,
        );

        self.dirty = false;
    }
}
