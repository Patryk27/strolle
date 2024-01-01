use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::{iter, mem};

use glam::Affine3A;

use crate::bvh::{Bvh, BvhNodeId};
use crate::primitive::Primitive;
use crate::primitives::BlasPrimitives;
use crate::utils::TriangleExt;
use crate::{Instance, Materials, Meshes, Params, Primitives, Triangles};

#[derive(Debug)]
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
    pub fn add(
        &mut self,
        instance_handle: P::InstanceHandle,
        instance: Instance<P>,
    ) {
        match self.instances.entry(instance_handle) {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut();

                entry.prev_xform = entry.instance.xform;
                entry.instance = instance;
                entry.dirty = true;
            }

            Entry::Vacant(entry) => {
                entry.insert(InstanceEntry {
                    prev_xform: instance.xform,
                    node_id: None,
                    dirty: true,
                    instance,
                });
            }
        }

        self.dirty = true;
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&P::InstanceHandle, &InstanceEntry<P>)> + Clone + '_
    {
        self.instances
            .iter()
            .map(|(instance_handle, instance_entry)| {
                (instance_handle, instance_entry)
            })
    }

    pub fn node_to_handle(&self, node: BvhNodeId) -> P::InstanceHandle {
        self.instances
            .iter()
            .find_map(|(key, val)| {
                // TODO use HashMap
                if val.node_id == Some(node) {
                    Some(*key)
                } else {
                    None
                }
            })
            .unwrap()
    }

    pub fn remove(
        &mut self,
        instance_handle: &P::InstanceHandle,
    ) -> Option<Instance<P>> {
        if let Some(instance) = self.instances.remove(instance_handle) {
            self.dirty = true;
            Some(instance.instance)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn refresh(
        &mut self,
        meshes: &mut Meshes<P>,
        materials: &Materials<P>,
        triangles: &mut Triangles<P>,
        primitives: &mut Primitives<P>,
        bvh: &mut Bvh,
    ) -> bool {
        if !mem::take(&mut self.dirty) {
            // return false;
        }

        for (instance_handle, entry) in &mut self.instances {
            let mesh_handle = entry.instance.mesh_handle;
            let material_handle = entry.instance.material_handle;

            if !mem::take(&mut entry.dirty) {
                // continue;
            }

            let Some(mesh) = meshes.get_mut(&mesh_handle) else {
                // If the mesh is not yet available, it might be still being
                // loaded in the background - in that case let's try again next
                // frame
                entry.dirty = true;
                self.dirty = true;
                continue;
            };

            let Some(material_id) = materials.lookup(&material_handle) else {
                // Same for materials
                entry.dirty = true;
                self.dirty = true;
                continue;
            };

            let triangle_ids = triangles.add(
                *instance_handle,
                mesh.triangles().iter().map(|triangle| {
                    triangle.build(
                        entry.instance.xform,
                        entry.instance.xform_inv_trans,
                    )
                }),
            );

            if entry.instance.inline {
                let tris = triangles.get(triangle_ids).map(
                    |(triangle_id, triangle)| Primitive::Triangle {
                        center: triangle.center(),
                        bounds: triangle.bounds(),
                        triangle_id,
                        material_id,
                    },
                );

                primitives.tlas_mut().add(*instance_handle, tris);
            } else {
                if let Some(node_id) = entry.node_id.take() {
                    bvh.delete_blas(node_id);
                }

                let bounds = mesh.bounds().with_transform(entry.instance.xform);

                let blas = primitives.create_blas(
                    *instance_handle,
                    BlasPrimitives::new(triangle_ids, bounds, material_id),
                );

                let node_id = bvh.refresh_blas(triangles, blas);

                primitives.tlas_mut().add(
                    *instance_handle,
                    iter::once(Primitive::Instance {
                        center: entry.instance.xform.translation.into(),
                        bounds: mesh
                            .bounds()
                            .with_transform(entry.instance.xform),
                        node_id,
                    }),
                );

                entry.node_id = Some(node_id);
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

#[derive(Debug)]
pub struct InstanceEntry<P>
where
    P: Params,
{
    pub instance: Instance<P>,
    pub node_id: Option<BvhNodeId>,
    pub prev_xform: Affine3A,
    pub dirty: bool,
}
