use crate::*;

/// Maps triangle vertices into UVs.
///
/// # Memory layout
///
/// One triangle's UVs take `3 [vertices] * 2 [f32 per vertice]` = `6 [f32]`,
/// which means that we can store two triangles worth of UVs in three `Vec4`,
/// giving us:
///
/// ```text
/// mapping #0              mapping #1
/// =====================   =====================
/// uvs[0]          uvs[1]          uvs[2]
/// -------------   -------------   -------------
/// x   y   z   w   x   y   z   w   x   y   z   w
/// .....   .....   .....   .....   .....   .....
/// uv0     uv1     uv2     uv0     uv1     uv2
/// ```
pub struct GeometryUvsView<'a> {
    _data: &'a [Vec4],
}

impl<'a> GeometryUvsView<'a> {
    pub fn new(data: &'a [Vec4]) -> Self {
        Self { _data: data }
    }

    pub fn get(&self, _: TriangleId) -> TriangleUv {
        // TODO
        Default::default()
    }
}
