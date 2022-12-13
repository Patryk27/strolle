use spirv_std::glam::Vec4;
use strolle_raytracer_models::{Triangle, TriangleId};

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
        self.len += 1;
    }

    pub fn get(&self, id: TriangleId) -> Triangle {
        let id = id.get() * 3;

        Triangle {
            v0: self.data[id],
            v1: self.data[id + 1],
            v2: self.data[id + 2],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (TriangleId, Triangle)> + '_ {
        (0..self.len)
            .map(TriangleId::new)
            .map(|id| (id, self.get(id)))
    }
}

impl StorageBufferable for GeometryTris {
    fn data(&self) -> &[u8] {
        bytemuck::cast_slice(&self.data)
    }
}
