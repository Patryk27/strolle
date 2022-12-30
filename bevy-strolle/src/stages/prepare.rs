use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use strolle as st;

use crate::state::{
    ExtractedInstances, ExtractedLights, ExtractedMaterials, ExtractedMeshes,
};
use crate::utils::color_to_vec4;
use crate::EngineRes;

pub(crate) fn meshes(
    mut engine: ResMut<EngineRes>,
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
                panic!("Mesh {:?} has no positions", mesh_handle);
            });

        let mesh_normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("Mesh {:?} has no normals", mesh_handle);
            });

        let mesh_indices: Vec<_> = mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("Mesh {:?} has no indices", mesh_handle);
            })
            .iter()
            .collect();

        let mesh_tris: Vec<_> = mesh_indices
            .chunks(3)
            .map(|vs| {
                let v0 = mesh_positions[vs[0]];
                let v1 = mesh_positions[vs[1]];
                let v2 = mesh_positions[vs[2]];

                let n0 = mesh_normals[vs[0]];
                let n1 = mesh_normals[vs[1]];
                let n2 = mesh_normals[vs[2]];

                st::Triangle::new(
                    vec3(v0[0], v0[1], v0[2]),
                    vec3(v1[0], v1[1], v1[2]),
                    vec3(v2[0], v2[1], v2[2]),
                    vec3(n0[0], n0[1], n0[2]),
                    vec3(n1[0], n1[1], n1[2]),
                    vec3(n2[0], n2[1], n2[2]),
                )
            })
            .collect();

        engine.add_mesh(mesh_handle, mesh_tris);
    }
}

pub(crate) fn materials(
    mut engine: ResMut<EngineRes>,
    mut materials: ResMut<ExtractedMaterials>,
) {
    for material_handle in materials.removed.iter() {
        engine.remove_material(material_handle);
    }

    for (material_handle, material) in materials.changed.drain(..) {
        let material = st::Material::default()
            .with_base_color(color_to_vec4(material.base_color))
            .with_perceptual_roughness(material.perceptual_roughness)
            .with_metallic(material.metallic)
            .with_reflectance(material.reflectance);

        engine.add_material(material_handle, material);
    }
}

pub(crate) fn instances(
    mut engine: ResMut<EngineRes>,
    mut instances: ResMut<ExtractedInstances>,
) {
    engine.clear_instances();

    for (mesh_handle, material_handle, transform) in instances.items.drain(..) {
        if !engine.contains_mesh(&mesh_handle) {
            // This can happen if this mesh is being loaded in the background -
            // in this case we can't instantiate it (yet), since we don't know
            // how it is going to look like.
            continue;
        }

        engine.add_instance(mesh_handle, material_handle, transform);
    }
}

pub(crate) fn lights(
    mut engine: ResMut<EngineRes>,
    mut lights: ResMut<ExtractedLights>,
) {
    engine.clear_lights();

    for (entity, light) in lights.items.drain(..) {
        engine.add_light(entity, light);
    }
}
