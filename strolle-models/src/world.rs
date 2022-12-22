use crate::*;

pub struct World<'a> {
    pub global_idx: u32,
    pub local_idx: u32,
    pub geometry_tris: GeometryTrisView<'a>,
    pub geometry_uvs: GeometryUvsView<'a>,
    pub geometry_bvh: GeometryBvhView<'a>,
    pub camera: &'a Camera,
    pub lights: &'a Lights,
    pub materials: &'a Materials,
}
