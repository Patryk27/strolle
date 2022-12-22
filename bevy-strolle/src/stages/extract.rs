use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::Extract;
use strolle as st;

use crate::state::ExtractedCamera;
use crate::utils::{color_to_vec3, color_to_vec4};
use crate::SyncedState;

#[allow(clippy::type_complexity)]
pub(crate) fn geometry(
    mut state: ResMut<SyncedState>,
    meshes: Extract<Res<Assets<Mesh>>>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
    models: Extract<
        Query<(
            Entity,
            &GlobalTransform,
            &Handle<Mesh>,
            &Handle<StandardMaterial>,
        )>,
    >,
    changed_models: Extract<
        Query<
            (
                Entity,
                &GlobalTransform,
                &Handle<Mesh>,
                &Handle<StandardMaterial>,
            ),
            Or<(
                Changed<Handle<Mesh>>,
                Changed<GlobalTransform>,
                Changed<Handle<StandardMaterial>>,
            )>,
        >,
    >,
    assets_rx: Extract<EventReader<AssetEvent<Mesh>>>,
) {
    if !state.is_active() {
        return;
    }

    if changed_models.is_empty() && assets_rx.is_empty() {
        return;
    }

    let state = &mut *state;

    // TODO we shouldn't have to extract the entire geometry each time
    state.geometry = Default::default();

    for (entity, transform, mesh, material) in models.iter() {
        let transform = transform.compute_matrix();
        let Some(mesh) = meshes.get(mesh) else { continue };
        let Some(material) = materials.get(material) else { continue };

        let material_id = {
            let material = st::Material::default()
                .with_base_color(color_to_vec4(material.base_color))
                .with_perceptual_roughness(material.perceptual_roughness)
                .with_metallic(material.metallic)
                .with_reflectance(material.reflectance);

            state.materials.alloc(entity, material)
        };

        // TODO we could support more, if we wanted
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("Entity {:?}'s mesh has no positions", entity);
            });

        let normals = mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("Entity {:?}'s mesh has no normals", entity);
            });

        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();

        let tris = indices.chunks(3).map(|vs| {
            let v0 = positions[vs[0]];
            let v1 = positions[vs[1]];
            let v2 = positions[vs[2]];

            let n0 = normals[vs[0]];
            let n1 = normals[vs[1]];
            let n2 = normals[vs[2]];

            st::Triangle::new(
                vec3(v0[0], v0[1], v0[2]),
                vec3(v1[0], v1[1], v1[2]),
                vec3(v2[0], v2[1], v2[2]),
                vec3(n0[0], n0[1], n0[2]),
                vec3(n1[0], n1[1], n1[2]),
                vec3(n2[0], n2[1], n2[2]),
                material_id,
            )
            .with_transform(transform)
        });

        for tri in tris {
            state.geometry.alloc(tri);
        }
    }

    state.geometry.reindex();
}

pub(crate) fn lights(
    mut state: ResMut<SyncedState>,
    lights: Extract<Query<(&PointLight, &GlobalTransform)>>,
) {
    if !state.is_active() {
        return;
    }

    let state = &mut *state;

    state.lights = Default::default();

    for (light, transform) in lights.iter() {
        let lum_intensity = light.intensity / (4.0 * PI);

        state.lights.push(st::Light::point(
            transform.translation(),
            color_to_vec3(light.color) * lum_intensity,
            light.range,
        ));
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn cameras(
    mut commands: Commands,
    default_clear_color: Option<Res<ClearColor>>,
    cameras: Extract<
        Query<(
            Entity,
            &Camera,
            &Camera3d,
            &CameraRenderGraph,
            &Projection,
            &GlobalTransform,
        )>,
    >,
) {
    for (
        entity,
        camera,
        camera_3d,
        camera_render_graph,
        projection,
        transform,
    ) in cameras.iter()
    {
        if !camera.is_active || **camera_render_graph != crate::graph::NAME {
            continue;
        }

        // TODO it feels like we should be able to reuse `.get_projection_matrix()`,
        //      but I can't come up with anything working at the moment
        let Projection::Perspective(projection) = projection else { continue };

        let clear_color = match &camera_3d.clear_color {
            ClearColorConfig::Default => default_clear_color
                .as_ref()
                .map(|cc| cc.0)
                .unwrap_or(Color::BLACK),
            ClearColorConfig::Custom(color) => *color,
            ClearColorConfig::None => {
                // TODO our camera doesn't support transparent clear color, so
                //      this is semi-invalid (as in: it works differently than
                //      in bevy_render)
                Color::rgba(0.0, 0.0, 0.0, 1.0)
            }
        };

        commands.get_or_spawn(entity).insert(ExtractedCamera {
            transform: *transform,
            projection: projection.clone(),
            clear_color,
        });
    }
}
