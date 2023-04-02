use crate::gpu;

#[derive(Clone, Copy, Debug)]
pub struct BvhTriangle {
    pub triangle: gpu::Triangle,
    pub triangle_id: gpu::TriangleId,
    pub material_id: gpu::MaterialId,
}
