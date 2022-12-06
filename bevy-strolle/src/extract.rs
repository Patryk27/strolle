use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::math::{vec3, vec4};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::Extract;
use strolle as st;

use crate::{ExtractedState, Synchronized};

type ModelComponents = (
    With<Transform>,
    With<Handle<Mesh>>,
    With<Handle<StandardMaterial>>,
);

type AddedModelComponents = (
    Added<Transform>,
    Added<Handle<Mesh>>,
    Added<Handle<StandardMaterial>>,
);

type ChangedModelComponents = (
    Changed<Transform>,
    Changed<Handle<Mesh>>,
    Changed<Handle<StandardMaterial>>,
);

pub(super) fn geometry(
    mut commands: Extract<Commands>,
    mut state: ResMut<ExtractedState>,
    meshes: Extract<Res<Assets<Mesh>>>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
    models: Extract<
        Query<(&Transform, &Handle<Mesh>, &Handle<StandardMaterial>)>,
    >,
    deleted_geo: Extract<RemovedComponents<Synchronized>>,
    created_geo: Extract<
        Query<Entity, (Or<AddedModelComponents>, ModelComponents)>,
    >,
    updated_geo: Extract<
        // TODO if only the material was changed, we don't have to rebuild the
        //      mesh
        //
        // TODO this should piggy-back on `Synchronized`, but seems not to work
        Query<Entity, (Or<ChangedModelComponents>, ModelComponents)>,
    >,
) {
    let state = &mut *state;

    for entity in deleted_geo.iter() {
        state.geometry.free(entity);
        state.materials.free(entity);
    }

    // -----

    for entity in created_geo.iter().chain(updated_geo.iter()) {
        let (transform, mesh, material) =
            models.get(entity).unwrap_or_else(|_| {
                panic!(
                    "Entity {:?} looks like a model, but it's missing some of \
                     the components we expect models to have - this is a bug \
                     in bevy-strolle",
                    entity
                );
            });

        let transform = transform.compute_matrix();
        let mesh = meshes.get(mesh).unwrap();
        let material = materials.get(material).unwrap();

        let material_id = {
            let material = st::Material::default()
                .with_base_color(color_to_vec4(material.base_color));

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

        let indices: Vec<_> = mesh.indices().unwrap().iter().collect();

        let tris = indices.chunks(3).map(|vs| {
            let v0 = positions[vs[0]];
            let v1 = positions[vs[1]];
            let v2 = positions[vs[2]];

            st::Triangle::new(
                vec3(v0[0], v0[1], v0[2]),
                vec3(v1[0], v1[1], v1[2]),
                vec3(v2[0], v2[1], v2[2]),
                material_id,
            )
            .with_transform(transform)
            .with_casts_shadows(true)
        });

        // TODO this can be done without allocating
        let tris: Vec<_> = tris.collect();

        if state.geometry.count(entity) == tris.len() {
            // It's a known object

            let mut tris = tris.iter();

            state.geometry.update(entity, || *tris.next().unwrap());
        } else {
            // It's a new object or the object's mesh had been changed
            state.geometry.free(entity);

            for tri in tris {
                // TODO misising feature
                let tri_uv = Default::default();

                state.geometry.alloc(entity, tri, tri_uv);
            }

            commands.entity(entity).insert(Synchronized);
        }
    }
}

pub(super) fn lights(
    mut state: ResMut<ExtractedState>,
    lights: Extract<Query<(&PointLight, &GlobalTransform)>>,
) {
    let state = &mut *state;

    state.lights = Default::default();

    for (light, transform) in lights.iter() {
        let lum_intensity = light.intensity / (4.0 * PI);

        state.lights.push(st::Light::point(
            transform.translation(),
            color_to_vec3(light.color) * lum_intensity,
        ));
    }
}

pub(super) fn camera(
    mut state: ResMut<ExtractedState>,
    default_clear_color: Option<Res<ClearColor>>,
    cameras: Extract<
        Query<(
            &Camera,
            &CameraRenderGraph,
            &Camera3d,
            &Projection,
            &GlobalTransform,
        )>,
    >,
) {
    let camera =
        cameras
            .iter()
            .find(|(camera, camera_render_graph, _, _, _)| {
                camera.is_active && ***camera_render_graph == crate::graph::NAME
            });

    let Some((camera, _, camera_3d, projection, transform)) = camera else { return };
    let size = camera.physical_viewport_size().unwrap();

    // TODO it feels like we should be able to reuse `.get_projection_matrix()`,
    //      but I can't come up with anything working at the moment
    let Projection::Perspective(projection) = projection else { return };

    let clear_color = match &camera_3d.clear_color {
        ClearColorConfig::Default => default_clear_color
            .map(|cc| cc.0.into())
            .unwrap_or(Color::BLACK),
        ClearColorConfig::Custom(color) => (*color).into(),
        ClearColorConfig::None => Color::BLACK,
    };

    state.camera = st::Camera::new(
        transform.translation(),
        transform.translation() + transform.forward(),
        transform.up(),
        size,
        projection.fov,
        color_to_vec3(clear_color),
    );
}

fn color_to_vec3(color: Color) -> Vec3 {
    let [r, g, b, _] = color.as_linear_rgba_f32();

    vec3(r, g, b)
}

fn color_to_vec4(color: Color) -> Vec4 {
    let [r, g, b, a] = color.as_linear_rgba_f32();

    vec4(r, g, b, a)
}
