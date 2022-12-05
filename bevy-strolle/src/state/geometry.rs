use crate::*;

pub struct Geometry {
    static_geo: Box<st::StaticGeometry>,
    static_geo_index: Option<Box<st::StaticGeometryIndex>>,
    static_geo_owners: Vec<Option<Entity>>,
    dynamic_geo: Box<st::DynamicGeometry>,
    dynamic_geo_owners: Vec<Entity>,
    uvs: Box<st::TriangleUvs>,
}

impl Geometry {
    pub fn alloc(
        &mut self,
        entity: Entity,
        tri: st::Triangle,
        tri_uv: st::TriangleUv,
    ) {
        log::trace!(
            "Allocating (dynamic): {:?} (tri={:?}, tri_uv={:?})",
            entity,
            tri,
            tri_uv
        );

        let id = self
            .dynamic_geo
            .push(tri)
            .expect("Tried to allocate too many dynamic triangles at once");

        self.dynamic_geo_owners.push(entity);
        self.uvs.set(id.into_any(), tri_uv);
    }

    pub fn free(&mut self, entity: Entity) {
        log::trace!("Freeing: {:?}", entity);

        self.free_dynamic(entity);
    }

    fn free_dynamic(&mut self, entity: Entity) {
        let mut id = 0;

        while id < self.dynamic_geo.len() {
            if self.dynamic_geo_owners[id] == entity {
                let tid = st::TriangleId::new_dynamic(id);

                self.dynamic_geo.remove(tid);
                self.dynamic_geo_owners.remove(id);
                self.uvs.remove(tid);
            } else {
                id += 1;
            }
        }
    }

    pub fn inner(
        &mut self,
    ) -> Option<(
        &st::StaticGeometry,
        &st::StaticGeometryIndex,
        &st::DynamicGeometry,
        &st::TriangleUvs,
    )> {
        if self.static_geo_index.is_none() {
            self.static_geo_index =
                st::GeometryIndexer::index(&self.static_geo);
        }

        Some((
            &self.static_geo,
            self.static_geo_index.as_ref()?,
            &self.dynamic_geo,
            &self.uvs,
        ))
    }
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            static_geo: Default::default(),
            static_geo_index: Default::default(),
            static_geo_owners: vec![None; st::MAX_STATIC_TRIANGLES],
            dynamic_geo: Default::default(),
            dynamic_geo_owners: Vec::with_capacity(st::MAX_DYNAMIC_TRIANGLES),
            uvs: Default::default(),
        }
    }
}