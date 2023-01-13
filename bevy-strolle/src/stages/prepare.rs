use std::collections::HashMap;
use std::time::Instant;

use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::PrimitiveTopology;
use strolle as st;

use crate::state::{
    ExtractedImages, ExtractedInstances, ExtractedLights, ExtractedMaterials,
    ExtractedMeshes,
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
                _ => None,
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_indices: Vec<_> = mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("Mesh {mesh_handle:?} has no indices");
            })
            .iter()
            .collect();

        let mesh_tris: Vec<_> = mesh_indices
            .chunks(3)
            .map(|vs| {
                let vertex0 = mesh_positions[vs[0]];
                let vertex1 = mesh_positions[vs[1]];
                let vertex2 = mesh_positions[vs[2]];

                let normal0 = mesh_normals[vs[0]];
                let normal1 = mesh_normals[vs[1]];
                let normal2 = mesh_normals[vs[2]];

                let uv0 = mesh_uvs.get(vs[0]).unwrap_or(&[0.0, 0.0]);
                let uv1 = mesh_uvs.get(vs[1]).unwrap_or(&[0.0, 0.0]);
                let uv2 = mesh_uvs.get(vs[2]).unwrap_or(&[0.0, 0.0]);

                st::Triangle::new(
                    vec3(vertex0[0], vertex0[1], vertex0[2]),
                    vec3(vertex1[0], vertex1[1], vertex1[2]),
                    vec3(vertex2[0], vertex2[1], vertex2[2]),
                    //
                    vec3(normal0[0], normal0[1], normal0[2]),
                    vec3(normal1[0], normal1[1], normal1[2]),
                    vec3(normal2[0], normal2[1], normal2[2]),
                    //
                    vec2(uv0[0], uv0[1]),
                    vec2(uv1[0], uv1[1]),
                    vec2(uv2[0], uv2[1]),
                )
            })
            .collect();

        engine.add_mesh(mesh_handle, mesh_tris);
    }
}

// TODO we should only load images that are used in materials
pub(crate) fn images(
    mut engine: ResMut<EngineResource>,
    mut images: ResMut<ExtractedImages>,
    image_assets: Res<RenderAssets<Image>>,
    mut pending_images: Local<HashMap<Handle<Image>, Instant>>,
) {
    for image_handle in images.removed.drain(..) {
        engine.remove_image(&image_handle);
        pending_images.remove(&image_handle);
    }

    // ---

    let mut completed_pending_images = Vec::new();

    for (image_handle, image_noticed_at) in pending_images.iter() {
        // If loading this image takes too long, let's bail out; I'm not sure
        // when exactly can this happen, but retrying extracting the same image
        // over and over again just feels kinda wrong
        if image_noticed_at.elapsed().as_secs() > 10 {
            log::error!(
                "Couldn't load image {:?}: GpuImage hasn't been available for too long",
                image_handle
            );

            completed_pending_images.push(image_handle.clone_weak());
            continue;
        }

        if let Some(image) = image_assets.get(image_handle) {
            completed_pending_images.push(image_handle.clone_weak());

            engine.add_image(
                image_handle.clone_weak(),
                image.texture_view.clone(),
                image.sampler.clone(),
            );
        }
    }

    for image_handle in completed_pending_images {
        log::debug!("Image {:?} extracted (late)", image_handle);

        pending_images.remove(&image_handle);
    }

    // ---

    for image_handle in images.changed.drain() {
        if let Some(image) = image_assets.get(&image_handle) {
            log::debug!("Image {:?} extracted", image_handle);

            engine.add_image(
                image_handle,
                image.texture_view.clone(),
                image.sampler.clone(),
            );
        } else {
            // This can happen when Bevy fails to load the asset¹ - in this case
            // we've gotta retry loading it next frame.
            //
            // Note that while this feels like a minor thing, it actually
            // happens pretty often in practice - e.g. the textures-example
            // triggers this case about 1/4 times on my machine.
            //
            // ¹ when the asset-extractor returns `PrepareAssetError::RetryNextUpdate`

            log::debug!("Couldn't extract image {:?}: GpuImage not available; will try again next frame", image_handle);

            pending_images.insert(image_handle, Instant::now());
        }
    }
}

pub(crate) fn materials<M>(
    mut engine: ResMut<EngineResource>,
    mut materials: ResMut<ExtractedMaterials<M>>,
) where
    M: MaterialLike,
{
    for material_handle in materials.removed.iter() {
        engine.remove_material(&M::map_handle(material_handle.clone_weak()));
    }

    for (material_handle, material) in materials.changed.drain(..) {
        engine.add_material(
            M::map_handle(material_handle),
            material.into_material(),
        );
    }
}

pub(crate) fn clear_instances(mut engine: ResMut<EngineResource>) {
    engine.clear_instances();
}

pub(crate) fn instances<M>(
    mut engine: ResMut<EngineResource>,
    mut instances: ResMut<ExtractedInstances<M>>,
) where
    M: MaterialLike,
{
    for (mesh_handle, material_handle, transform) in instances.items.drain(..) {
        if !engine.contains_mesh(&mesh_handle) {
            // This can happen if this mesh is being loaded in the background -
            // in this case we can't instantiate it (yet), since we don't know
            // how it is going to look like.
            continue;
        }

        engine.add_instance(
            mesh_handle,
            M::map_handle(material_handle),
            transform,
        );
    }
}

pub(crate) fn lights(
    mut engine: ResMut<EngineResource>,
    mut lights: ResMut<ExtractedLights>,
) {
    engine.clear_lights();

    for (entity, light) in lights.items.drain(..) {
        engine.add_light(entity, light);
    }
}
