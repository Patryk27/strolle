use crate::*;

#[derive(Default)]
pub struct Geometry {
    tris: st::GeometryTris,
    uvs: st::GeometryUvs,
    bvh: st::GeometryBvh,
}

impl Geometry {
    pub fn alloc(&mut self, tri: st::Triangle) {
        self.tris.push(tri);
    }

    pub fn reindex(&mut self) {
        self.bvh.rebuild(&self.tris);
    }

    pub fn inner(
        &self,
    ) -> (&st::GeometryTris, &st::GeometryUvs, &st::GeometryBvh) {
        (&self.tris, &self.uvs, &self.bvh)
    }
}
