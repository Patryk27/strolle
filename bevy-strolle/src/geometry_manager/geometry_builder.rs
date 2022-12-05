use bevy::math::vec3;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;

use crate::*;

pub struct GeometryBuilder<'a> {
    geo: &'a mut GeometryManager,
}

impl<'a> GeometryBuilder<'a> {
    pub(super) fn new(geo: &'a mut GeometryManager) -> Self {
        Self { geo }
    }

    pub fn add(&mut self, entity: Entity, mesh: &Mesh, transform: Mat4) {
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap();

        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();

        let tris = indices.chunks(3).map(|vs| {
            let v0 = positions[vs[0]];
            let v1 = positions[vs[1]];
            let v2 = positions[vs[2]];

            st::Triangle::new(
                vec3(v0[0], v0[1], v0[2]),
                vec3(v1[0], v1[1], v1[2]),
                vec3(v2[0], v2[1], v2[2]),
                st::MaterialId::new(0),
            )
            .with_alpha(1.0)
            .with_transform(transform)
            .with_casts_shadows(true)
            .with_uv_transparency(false)
            .with_double_sided(true)
            .with_uv_divisor(1, 1)
        });

        for tri in tris {
            self.geo.alloc_dynamic(entity, tri, Default::default());
        }
    }
}
