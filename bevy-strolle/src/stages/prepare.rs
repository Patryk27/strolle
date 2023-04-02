use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{
    ExtractedInstances, ExtractedLights, ExtractedMaterials, ExtractedMeshes,
};
use crate::{EngineResource, MaterialLike};

pub(crate) fn meshes(
    mut engine: ResMut<EngineResource>,
    mut meshes: ResMut<ExtractedMeshes>,
) {
    for mesh_handle in meshes
        .removed
        .iter()
        .chain(meshes.changed.iter().map(|(k, _)| k))
    {
        engine.remove_mesh(mesh_handle);
    }

    for (mesh_handle, mesh) in meshes.changed.drain(..) {
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);

        let mesh_positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("Mesh {mesh_handle:?} has no positions");
            });

        let mesh_normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("Mesh {mesh_handle:?} has no normals");
            });

        let mesh_uvs = mesh
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .and_then(|uvs| match uvs {
                VertexAttributeValues::Float32x2(uvs) => Some(uvs),
                _ => panic!(
                    "Mesh {mesh_handle:?} uses unsupported format for UVs"
                ),
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_tans = mesh
            .attribute(Mesh::ATTRIBUTE_TANGENT)
            .and_then(|uvs| match uvs {
                VertexAttributeValues::Float32x4(tangents) => Some(tangents),
                _ => panic!(
                    "Mesh {mesh_handle:?} uses unsupported format for tangents"
                ),
            })
            .map(|tangents| tangents.as_slice())
            .unwrap_or(&[]);

        let mesh_indices: Vec<_> = mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("Mesh {mesh_handle:?} has no indices");
            })
            .iter()
            .collect();

        let mesh_triangles: Vec<_> = mesh_indices
            .chunks(3)
            .map(|vs| {
                let position0 = mesh_positions[vs[0]];
                let position1 = mesh_positions[vs[1]];
                let position2 = mesh_positions[vs[2]];

                let normal0 = mesh_normals[vs[0]];
                let normal1 = mesh_normals[vs[1]];
                let normal2 = mesh_normals[vs[2]];

                let uv0 = mesh_uvs.get(vs[0]).copied().unwrap_or_default();
                let uv1 = mesh_uvs.get(vs[1]).copied().unwrap_or_default();
                let uv2 = mesh_uvs.get(vs[2]).copied().unwrap_or_default();

                let tan0 = mesh_tans.get(vs[0]).copied().unwrap_or_default();
                let tan1 = mesh_tans.get(vs[1]).copied().unwrap_or_default();
                let tan2 = mesh_tans.get(vs[2]).copied().unwrap_or_default();

                st::Triangle::default()
                    .with_positions([position0, position1, position2])
                    .with_normals([normal0, normal1, normal2])
                    .with_uvs([uv0, uv1, uv2])
                    .with_tangents([tan0, tan1, tan2])
            })
            .collect();

        engine.add_mesh(mesh_handle, st::Mesh::new(mesh_triangles));
    }
}

pub(crate) fn materials<M>(
    mut engine: ResMut<EngineResource>,
    mut materials: ResMut<ExtractedMaterials<M>>,
    image_assets: Res<RenderAssets<Image>>,
    mut pending_images: Local<HashSet<Handle<Image>>>,
) where
    M: MaterialLike,
{
    pending_images.drain_filter(|image_handle| {
        if let Some(image) = image_assets.get(image_handle) {
            engine.add_image(
                image_handle.clone(),
                image.texture_view.clone(),
                image.sampler.clone(),
            );

            true
        } else {
            false
        }
    });

    for material_handle in materials.removed.iter() {
        engine.remove_material(&M::map_handle(material_handle.clone_weak()));
    }

    for (material_handle, material) in materials.changed.drain(..) {
        for image_handle in material.images() {
            if engine.has_image(image_handle) {
                continue;
            }

            if let Some(image) = image_assets.get(image_handle) {
                engine.add_image(
                    image_handle.clone(),
                    image.texture_view.clone(),
                    image.sampler.clone(),
                );

                pending_images.remove(image_handle);
            } else {
                pending_images.insert(image_handle.clone());
            }
        }

        engine.add_material(
            M::map_handle(material_handle),
            material.into_material(),
        );
    }
}

pub(crate) fn instances<M>(
    mut engine: ResMut<EngineResource>,
    mut instances: ResMut<ExtractedInstances<M>>,
) where
    M: MaterialLike,
{
    for (entity, mesh_handle, material_handle, transform) in
        instances.changed.drain(..)
    {
        let material_handle = M::map_handle(material_handle);

        engine.add_instance(
            entity,
            st::Instance::new(mesh_handle, material_handle, transform),
        );
    }

    for entity in instances.removed.drain(..) {
        engine.remove_instance(&entity);
    }
}

pub(crate) fn lights(
    mut engine: ResMut<EngineResource>,
    mut lights: ResMut<ExtractedLights>,
) {
    engine.remove_all_lights();

    for (entity, light) in lights.items.drain(..) {
        engine.add_light(entity, light);
    }
}
