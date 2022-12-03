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

        let tris = positions.chunks_exact(3).map(|vs| {
            st::Triangle::new(
                vec3(vs[0][0], vs[0][1], vs[0][2]),
                vec3(vs[1][0], vs[1][1], vs[1][2]),
                vec3(vs[2][0], vs[2][1], vs[2][2]),
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
