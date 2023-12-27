use std::mem;

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::PrimitiveTopology;
use strolle as st;

use crate::state::{
    ExtractedImageData, ExtractedImages, ExtractedInstances, ExtractedLights,
    ExtractedMaterials, ExtractedMeshes, ExtractedSun,
};
use crate::utils::GlamCompat;
use crate::{EngineResource, MaterialLike};

pub(crate) fn meshes(
    mut engine: ResMut<EngineResource>,
    mut meshes: ResMut<ExtractedMeshes>,
) {
    for mesh_id in meshes
        .removed
        .iter()
        .chain(meshes.changed.iter().map(|mesh| &mesh.id))
    {
        engine.remove_mesh(mesh_id);
    }

    for mesh in mem::take(&mut meshes.changed) {
        // HACK
        if mesh.mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            continue;
        }

        let mesh_positions = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no positions", mesh.id);
            });

        let mesh_normals = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no normals", mesh.id);
            });

        let mesh_uvs = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x2(uvs) => uvs,
                _ => {
                    panic!("mesh {:?} has unsupported format for UVs", mesh.id)
                }
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_tans = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_TANGENT)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x4(tangents) => tangents,
                _ => panic!(
                    "mesh {:?} has unsupported format for tangents",
                    mesh.id
                ),
            })
            .map(|tangents| tangents.as_slice())
            .unwrap_or(&[]);

        let mesh_indices: Vec<_> = mesh
            .mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no indices", mesh.id);
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

                st::MeshTriangle::default()
                    .with_positions([position0, position1, position2])
                    .with_normals([normal0, normal1, normal2])
                    .with_uvs([uv0, uv1, uv2])
                    .with_tangents([tan0, tan1, tan2])
            })
            .collect();

        engine.add_mesh(mesh.id, st::Mesh::new(mesh_triangles));
    }
}

pub(crate) fn materials<M>(
    mut engine: ResMut<EngineResource>,
    mut materials: ResMut<ExtractedMaterials<M>>,
) where
    M: MaterialLike,
{
    for material_id in materials.removed.iter() {
        engine.remove_material(&M::map_id(*material_id));
    }

    for material in materials.changed.drain(..) {
        engine.add_material(
            M::map_id(material.id),
            material.material.into_material(),
        );
    }
}

pub(crate) fn images(
    mut engine: ResMut<EngineResource>,
    textures: Res<RenderAssets<Image>>,
    mut images: ResMut<ExtractedImages>,
) {
    for image_handle in &images.removed {
        engine.remove_image(image_handle);
    }

    for image in mem::take(&mut images.changed) {
        // HACK because we .add_image() all images we can find (instead of
        //      making sure to load only images used by any material), we
        //      unavoidably stumble upon some 1D / 3D images that Bevy (or
        //      something?) preloads for some internal reasons
        //
        //      bottom line is:
        //      this condition shouldn't be necessary if we realize the "load
        //      only images used in materials" todo below
        if image.texture_descriptor.dimension != wgpu::TextureDimension::D2 {
            continue;
        }

        let data = match image.data {
            ExtractedImageData::Raw { data } => st::ImageData::Raw { data },

            ExtractedImageData::Texture { is_dynamic } => {
                st::ImageData::Texture {
                    texture: textures.get(image.id).unwrap().texture.clone(),
                    is_dynamic,
                }
            }
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
            image.id,
            st::Image::new(
                data,
                image.texture_descriptor,
                image.sampler_descriptor,
            ),
        );
    }
}

pub(crate) fn instances<M>(
    mut engine: ResMut<EngineResource>,
    mut instances: ResMut<ExtractedInstances<M>>,
) where
    M: MaterialLike,
{
    for instance_handle in mem::take(&mut instances.removed) {
        engine.remove_instance(&instance_handle);
    }

    for instance in mem::take(&mut instances.changed) {
        engine.add_instance(
            instance.id,
            st::Instance::new(
                instance.mesh_id,
                M::map_id(instance.material_id),
                instance.xform.compat(),
            ),
        );
    }
}

pub(crate) fn lights(
    mut engine: ResMut<EngineResource>,
    mut lights: ResMut<ExtractedLights>,
) {
    for light_handle in &lights.removed {
        engine.remove_light(light_handle);
    }

    for light in mem::take(&mut lights.changed) {
        engine.add_light(light.id, light.light);
    }
}

pub(crate) fn sun(
    mut engine: ResMut<EngineResource>,
    mut sun: ResMut<ExtractedSun>,
) {
    if let Some(sun) = sun.sun.take() {
        engine.update_sun(sun);
    }
}
