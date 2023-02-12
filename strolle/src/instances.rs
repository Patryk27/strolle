use std::collections::HashMap;
use std::mem;

use crate::meshes::Meshes;
use crate::triangles::Triangles;
use crate::{Instance, Params};

#[derive(Debug)]
pub struct Instances<P>
where
    P: Params,
{
    instances: HashMap<P::InstanceHandle, (Instance<P>, bool)>,
    dirty: bool,
}

impl<P> Instances<P>
where
    P: Params,
{
    pub fn add(
        &mut self,
        instance_handle: P::InstanceHandle,
        instance: Instance<P>,
    ) {
        self.instances.insert(instance_handle, (instance, true));
        self.dirty = true;
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&P::InstanceHandle, &Instance<P>)> + Clone + '_
    {
        self.instances
            .iter()
            .map(|(instance_handle, (instance, _))| (instance_handle, instance))
    }

    pub fn remove(&mut self, instance_handle: &P::InstanceHandle) {
        self.instances.remove(instance_handle);
        self.dirty = true;
    }

    pub fn refresh(
        &mut self,
        meshes: &Meshes<P>,
        triangles: &mut Triangles<P>,
    ) -> bool {
        if !mem::take(&mut self.dirty) {
            return false;
        }

        for (instance_handle, (instance, dirty)) in &mut self.instances {
            if !mem::take(dirty) {
                continue;
            }

            let Some(mesh) = meshes.get(instance.mesh_handle()) else {
                // If the mesh is not yet available, it might be still being
                // loaded in the background - in that case let's try again next
                // frame:
                *dirty = true;
                self.dirty = true;
                continue;
            };

            let mesh_triangles = mesh
                .triangles()
                .iter()
                .map(|triangle| triangle.transformed(instance.transform()));

            if let Some(count) = triangles.count(instance_handle) {
                if mesh.triangles().len() == count {
                    triangles.update(instance_handle, mesh_triangles);
                } else {
                    triangles.remove(instance_handle);
                    triangles.add(instance_handle.to_owned(), mesh_triangles);
                }
            } else {
                triangles.add(instance_handle.to_owned(), mesh_triangles);
            }
        }

        true
    }
}

impl<P> Default for Instances<P>
where
    P: Params,
{
    fn default() -> Self {
        Self {
            instances: Default::default(),
            dirty: Default::default(),
        }
    }
}
