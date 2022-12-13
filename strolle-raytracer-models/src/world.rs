use crate::*;

pub struct World<'a> {
    pub geometry_tris: GeometryTrisView<'a>,
    pub geometry_uvs: GeometryUvsView<'a>,
    pub geometry_bvh: GeometryBvhView<'a>,
    pub camera: &'a Camera,
    pub lights: &'a Lights,
    pub materials: &'a Materials,
}
