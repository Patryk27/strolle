use crate::*;

pub struct GeometryTrisView<'a> {
    data: &'a [Vec4],
}

impl<'a> GeometryTrisView<'a> {
    pub fn new(data: &'a [Vec4]) -> Self {
        Self { data }
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        let id = id.get() * 6;

        // TODO safety

        let v0 = unsafe { *self.data.get_unchecked(id) };
        let v1 = unsafe { *self.data.get_unchecked(id + 1) };
        let v2 = unsafe { *self.data.get_unchecked(id + 2) };

        let n0 = unsafe { *self.data.get_unchecked(id + 3) };
        let n1 = unsafe { *self.data.get_unchecked(id + 4) };
        let n2 = unsafe { *self.data.get_unchecked(id + 5) };

        Triangle {
            v0,
            v1,
            v2,
            n0,
            n1,
            n2,
        }
    }
}
