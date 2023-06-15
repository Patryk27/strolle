use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::ImageSampler;
use strolle as st;

use crate::state::{
    ExtractedImages, ExtractedInstances, ExtractedLights, ExtractedMaterials,
    ExtractedMeshes,
};
use crate::utils::GlamCompat;
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
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x2(uvs) => uvs,
                _ => panic!(
                    "Mesh {mesh_handle:?} uses unsupported format for UVs"
                ),
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_tans = mesh
            .attribute(Mesh::ATTRIBUTE_TANGENT)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x4(tangents) => tangents,
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

pub(crate) fn images(
    mut engine: ResMut<EngineResource>,
    mut images: ResMut<ExtractedImages>,
) {
    for image_handle in images.removed.iter() {
        engine.remove_image(image_handle);
    }

    for (image_handle, image) in images.changed.drain(..) {
        // HACK because we .add_image() all images we can find instead of making
        //      sure to load only images used by any material, we unavoidably
        //      stumble upon some 1D / 3D images that Bevy (or something?)
        //      preloads
        //
        //      bottom line is:
        //      this condition shouldn't be necessary if we realize the "load
        //      only images used in materials" todo below
        if image.texture_descriptor.dimension != wgpu::TextureDimension::D2 {
            continue;
        }

        let sampler_descriptor = match image.sampler_descriptor {
            ImageSampler::Default => {
                // TODO as per Bevy's docs, this should actually read the
                //      defaults as specified in the `ImagePlugin`'s setup
                ImageSampler::nearest_descriptor()
            }

            ImageSampler::Descriptor(descriptor) => descriptor,
        };

        // TODO we should add only those images which are used by at least one
        //      material, since otherwise we'll .add_image() textures that are
        //      related solely to UI, for instance
        //
        //      (conversely, we should remove those images which are not in use
        //      by any material)
        //
        //      that's not so easy though because it can happen that an image is
        //      loaded first *and then* (e.g. in next frame) it's used by some
        //      material, in which case a simple condition right here will not
        //      be sufficient
        engine.add_image(
            image_handle,
            image.data,
            image.texture_descriptor,
            sampler_descriptor,
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
            st::Instance::new(mesh_handle, material_handle, transform.compat()),
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
