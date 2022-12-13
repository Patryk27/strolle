use std::collections::HashSet;

use crate::*;

pub struct Materials {
    materials: Box<st::Materials>,
    owners: Vec<HashSet<Entity>>,
}

impl Materials {
    pub fn alloc(
        &mut self,
        entity: Entity,
        material: st::Material,
    ) -> st::MaterialId {
        for id in 0..st::MAX_MATERIALS {
            let id = st::MaterialId::new(id);

            if self.materials.get(id) == material {
                // TODO remove `entity` as owner from the previous material
                self.owners[id.get()].insert(entity);
                return id;
            }
        }

        for id in 0..st::MAX_MATERIALS {
            if self.owners[id].contains(&entity) {
                if self.owners[id].len() == 1 {
                    let id = st::MaterialId::new(id);

                    self.materials.set(id, material);

                    return id;
                } else {
                    self.owners[id].remove(&entity);
                }
            }
        }

        log::trace!("Allocating: {:?} (material={:?})", entity, material);

        let id = (0..st::MAX_MATERIALS)
            .map(st::MaterialId::new)
            .find(|id| self.owners[id.get()].is_empty())
            .expect("Tried to allocate too many materials at once");

        self.materials.set(id, material);
        self.owners[id.get()].insert(entity);

        id
    }

    pub fn inner(&self) -> &st::Materials {
        &self.materials
    }
}

impl Default for Materials {
    fn default() -> Self {
        Self {
            materials: Default::default(),
            owners: vec![Default::default(); st::MAX_MATERIALS],
        }
    }
}
