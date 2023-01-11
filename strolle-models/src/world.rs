use crate::{
    BvhView, Info, InstancesView, LightsView, MaterialsView, TrianglesView,
};

pub struct World<'a> {
    pub global_idx: u32,
    pub local_idx: u32,
    pub triangles: TrianglesView<'a>,
    pub instances: InstancesView<'a>,
    pub bvh: BvhView<'a>,
    pub lights: LightsView<'a>,
    pub materials: MaterialsView<'a>,
    pub info: &'a Info,
}
