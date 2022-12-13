use crate::*;

pub struct GeometryTrisView<'a> {
    data: &'a [Vec4],
}

impl<'a> GeometryTrisView<'a> {
    pub fn new(data: &'a [Vec4]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        let id = id.get() * 3;

        // TODO safety
        let v0 = unsafe { *self.data.get_unchecked(id) };
        let v1 = unsafe { *self.data.get_unchecked(id + 1) };
        let v2 = unsafe { *self.data.get_unchecked(id + 2) };

        Triangle { v0, v1, v2 }
    }
}
