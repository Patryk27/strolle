use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::{iter, mem};

use glam::Affine3A;

use crate::bvh::Bvh;
use crate::primitive::Primitive;
use crate::primitives::{PrimitiveOwner, PrimitiveScope};
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
            return false;
        }

        for (instance_handle, entry) in &mut self.instances {
            let mesh_handle = entry.instance.mesh_handle;
            let material_handle = entry.instance.material_handle;

            if !mem::take(&mut entry.dirty) {
                continue;
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

            if entry.instance.inline {
                let tris = triangles.copy(
                    PrimitiveOwner::Mesh(mesh_handle),
                    PrimitiveOwner::Instance(*instance_handle),
                );

                let prims = tris.map(|(triangle_id, triangle)| {
                    *triangle = triangle.with_xform(
                        entry.instance.xform,
                        entry.instance.xform_inv_trans,
                    );

                    Primitive::Triangle {
                        center: triangle.center(),
                        bounds: triangle.bounds(),
                        triangle_id,
                        material_id,
                        inline: true,
                    }
                });

                primitives
                    .scope_mut(PrimitiveScope::Tlas)
                    .add(PrimitiveOwner::Instance(*instance_handle), prims);
            } else {
                let node_id = if let Some(node_id) = mesh.node_id() {
                    node_id
                } else {
                    let mesh_primitives = triangles
                        .get(PrimitiveOwner::Mesh(mesh_handle))
                        .map(|(triangle_id, triangle)| Primitive::Triangle {
                            triangle_id,
                            material_id,
                            center: triangle.center(),
                            bounds: triangle.bounds(),
                            inline: false,
                        });

                    primitives
                        .create_scope(PrimitiveScope::Blas(mesh_handle))
                        .add(
                            PrimitiveOwner::Mesh(mesh_handle),
                            mesh_primitives,
                        );

                    let node_id = bvh.create_blas(primitives, &mesh_handle);

                    *mesh.node_id_mut() = Some(node_id);

                    node_id
                };

                primitives.scope_mut(PrimitiveScope::Tlas).add(
                    PrimitiveOwner::Instance(*instance_handle),
                    iter::once(Primitive::Instance {
                        center: entry.instance.xform.translation.into(),
                        bounds: mesh
                            .bounds()
                            .with_transform(entry.instance.xform),
                        xform_inv: entry.instance.xform_inv,
                        node_id,
                    }),
                );
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
    pub prev_xform: Affine3A,
    pub dirty: bool,
}
