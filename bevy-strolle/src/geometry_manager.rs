mod geometry_builder;

pub use self::geometry_builder::*;
use crate::*;

pub struct GeometryManager {
    pub static_geo: Box<st::StaticGeometry>,
    pub static_geo_index: Option<Box<st::StaticGeometryIndex>>,
    static_geo_owners: Vec<Option<Entity>>,
    pub dynamic_geo: Box<st::DynamicGeometry>,
    dynamic_geo_owners: Vec<Entity>,
    pub uvs: Box<st::TriangleUvs>,
}

impl GeometryManager {
    pub fn builder(&mut self) -> GeometryBuilder<'_> {
        GeometryBuilder::new(self)
    }

    fn alloc_static(
        &mut self,
        entity: Entity,
        tri: st::Triangle,
        tri_uv: st::TriangleUv,
    ) {
        log::trace!(
            "Allocating (static): {:?} (tri={:?}, tri_uv={:?})",
            entity,
            tri,
            tri_uv
        );

        let id = (0..st::MAX_STATIC_TRIANGLES)
            .map(st::TriangleId::new_static)
            .find(|id| self.static_geo_owners[id.get()].is_none())
            .expect("Tried to allocate too many static triangles at once");

        self.static_geo.set(id, tri);
        self.static_geo_index = None;
        self.static_geo_owners[id.get()] = Some(entity);
        self.uvs.set(id.into_any(), tri_uv);
    }

    fn alloc_dynamic(
        &mut self,
        entity: Entity,
        tri: st::Triangle,
        tri_uv: st::TriangleUv,
    ) {
        let id = self
            .dynamic_geo
            .push(tri)
            .expect("Tried to allocate too many dynamic triangles at once");

        log::trace!(
            "Allocating (dynamic): {:?} (id={id:?}, tri={:?}, tri_uv={:?})",
            entity,
            tri,
            tri_uv
        );

        self.dynamic_geo_owners.push(entity);
        self.uvs.set(id.into_any(), tri_uv);
    }

    fn update_dynamic(
        &mut self,
        entity: Entity,
        mut next_tri: impl FnMut() -> (st::Triangle, st::TriangleUv),
    ) {
        for id in 0..self.dynamic_geo.len() {
            if self.dynamic_geo_owners[id] == entity {
                let tri_id = st::TriangleId::new_dynamic(id);

                let (tri, tri_uv) = next_tri();

                *self.dynamic_geo.get_mut(tri_id) = tri;
                self.uvs.set(tri_id.into_any(), tri_uv);
            }
        }
    }

    pub fn free(&mut self, entity: Entity) {
        log::trace!("Freeing: {:?}", entity);

        self.free_static(entity);
        self.free_dynamic(entity);
    }

    fn free_static(&mut self, entity: Entity) {
        let mut is_disty = false;

        for id in 0..st::MAX_STATIC_TRIANGLES {
            if self.static_geo_owners[id] == Some(entity) {
                self.static_geo
                    .set(st::TriangleId::new_static(id), Default::default());

                self.static_geo_owners[id] = None;

                is_disty = true;
            }
        }

        if is_disty {
            self.static_geo_index = None;
        }
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

impl Default for GeometryManager {
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
