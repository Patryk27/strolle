use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

use bevy::ecs::system::Resource;
use glam::Affine3A;
use rand::Rng;

use crate::bvh::Bvh;
use crate::materials::Materials;
use crate::meshes::Meshes;
use crate::triangles::Triangles;
use crate::{Instance, InstanceHandle};

#[derive(Debug, Default, Resource)]
pub struct Instances {
    instances: HashMap<InstanceHandle, InstanceEntry>,
    dirty: bool,
}

impl Instances {
    pub fn add(&mut self, handle: InstanceHandle, instance: Instance) {
        match self.instances.entry(handle) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();

                entry.prev_transform = entry.instance.transform;
                entry.instance = instance;
                entry.dirty = true;
            }

            Entry::Vacant(entry) => {
                entry.insert(InstanceEntry {
                    prev_transform: instance.transform,
                    uuid: rand::thread_rng().gen(),
                    dirty: true,
                    instance,
                });
            }
        }

        self.dirty = true;
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (InstanceHandle, &InstanceEntry)> + Clone + '_
    {
        self.instances
            .iter()
            .map(|(handle, entry)| (*handle, entry))
    }

    pub fn remove(&mut self, handle: InstanceHandle) {
        self.dirty |= self.instances.remove(&handle).is_some();
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn refresh(
        &mut self,
        meshes: &Meshes,
        materials: &Materials,
        triangles: &mut Triangles,
        bvh: &mut Bvh,
    ) -> bool {
        if !mem::take(&mut self.dirty) {
            return false;
        }

        for (&handle, entry) in &mut self.instances {
            if !mem::take(&mut entry.dirty) {
                continue;
            }

            let Some(mesh) = meshes.get(entry.instance.mesh_handle) else {
                // If the mesh is not yet available, it might be still being
                // loaded in the background - in that case let's try again next
                // frame
                entry.dirty = true;
                self.dirty = true;
                continue;
            };

            let Some(material_id) =
                materials.lookup(entry.instance.material_handle)
            else {
                // Same for materials
                entry.dirty = true;
                self.dirty = true;
                continue;
            };

            let mesh_triangles = mesh.triangles().iter().map(|triangle| {
                triangle.build(
                    entry.instance.transform,
                    entry.instance.transform_inverse,
                )
            });

            if let Some(count) = triangles.count(handle) {
                if mesh.triangles().len() == count {
                    triangles.update(bvh, handle, mesh_triangles, material_id);
                } else {
                    triangles.remove(bvh, handle);

                    triangles.add(
                        bvh,
                        handle.to_owned(),
                        mesh_triangles,
                        material_id,
                    );
                }
            } else {
                triangles.add(bvh, handle, mesh_triangles, material_id);
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct InstanceEntry {
    pub instance: Instance,
    pub uuid: u32,
    pub prev_transform: Affine3A,
    pub dirty: bool,
}
