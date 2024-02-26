use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

use derivative::Derivative;
use glam::Affine3A;
use rand::Rng;

use crate::bvh::Bvh;
use crate::materials::Materials;
use crate::meshes::Meshes;
use crate::triangles::Triangles;
use crate::{Instance, Params};

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Instances<P>
where
    P: Params,
{
    instances: HashMap<P::InstanceHandle, InstanceEntry<P>>,
    dirty: bool,
}

impl<P> Instances<P>
where
    P: Params,
{
    pub fn insert(&mut self, handle: P::InstanceHandle, item: Instance<P>) {
        match self.instances.entry(handle) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();

                entry.prev_transform = entry.instance.transform;
                entry.instance = item;
                entry.dirty = true;
            }

            Entry::Vacant(entry) => {
                entry.insert(InstanceEntry {
                    prev_transform: item.transform,
                    uuid: rand::thread_rng().gen(),
                    dirty: true,
                    instance: item,
                });
            }
        }

        self.dirty = true;
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (P::InstanceHandle, &InstanceEntry<P>)> + Clone + '_
    {
        self.instances
            .iter()
            .map(|(handle, entry)| (*handle, entry))
    }

    pub fn remove(&mut self, handle: P::InstanceHandle) {
        self.dirty |= self.instances.remove(&handle).is_some();
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn refresh(
        &mut self,
        meshes: &Meshes<P>,
        materials: &Materials<P>,
        triangles: &mut Triangles<P>,
        bvh: &mut Bvh,
    ) -> bool {
        if !mem::take(&mut self.dirty) {
            return false;
        }

        for (&instance_handle, entry) in &mut self.instances {
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

            if let Some(count) = triangles.count(instance_handle) {
                if mesh.triangles().len() == count {
                    triangles.update(
                        bvh,
                        instance_handle,
                        mesh_triangles,
                        material_id,
                    );
                } else {
                    triangles.remove(bvh, instance_handle);

                    triangles.create(
                        bvh,
                        instance_handle.to_owned(),
                        mesh_triangles,
                        material_id,
                    );
                }
            } else {
                triangles.create(
                    bvh,
                    instance_handle.to_owned(),
                    mesh_triangles,
                    material_id,
                );
            }
        }

        true
    }
}

#[derive(Debug)]
pub struct InstanceEntry<P>
where
    P: Params,
{
    pub instance: Instance<P>,
    pub uuid: u32,
    pub prev_transform: Affine3A,
    pub dirty: bool,
}
