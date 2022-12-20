use spirv_std::glam::Vec4;
use strolle_models::{Triangle, TriangleId};

use crate::StorageBufferable;

#[derive(Clone, Debug, Default)]
pub struct GeometryTris {
    data: Vec<Vec4>,
    len: usize,
}

impl GeometryTris {
    pub fn push(&mut self, tri: Triangle) {
        self.data.push(tri.v0);
        self.data.push(tri.v1);
        self.data.push(tri.v2);
        self.data.push(tri.n0);
        self.data.push(tri.n1);
        self.data.push(tri.n2);
        self.len += 1;
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        let id = id.get() * 6;

        Triangle {
            v0: self.data[id],
            v1: self.data[id + 1],
            v2: self.data[id + 2],
            n0: self.data[id + 3],
            n1: self.data[id + 4],
            n2: self.data[id + 5],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (TriangleId, Triangle)> + '_ {
        (0..self.len)
            .map(TriangleId::new)
            .map(|id| (id, self.get(id)))
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl StorageBufferable for GeometryTris {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
