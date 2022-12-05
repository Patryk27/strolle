use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::Extract;
use strolle as st;

use crate::ExtractedState;

pub(super) fn geometry(
    mut state: ResMut<ExtractedState>,
    meshes: Extract<Res<Assets<Mesh>>>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
    models: Extract<
        Query<(Entity, &Transform, &Handle<Mesh>, &Handle<StandardMaterial>)>,
    >,
) {
    let state = &mut *state;

    // TODO
    state.geometry = Default::default();

    for (entity, &transform, mesh, material) in models.iter() {
        let transform = transform.compute_matrix();
        let mesh = meshes.get(mesh).unwrap();

        let material_id = {
            let material = materials.get(material).unwrap();

            state.materials.alloc(
                entity,
                st::Material::default()
                    .with_color(color_to_vec3(material.base_color)),
            )
        };

        // TODO we could support more, if we wanted
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);

        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap();

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
            .with_alpha(1.0)
            .with_transform(transform)
            .with_casts_shadows(true)
            .with_uv_transparency(false)
            .with_double_sided(true)
            .with_uv_divisor(1, 1)
        });

        for tri in tris {
            // TODO
            let tri_uv = Default::default();

            state.geometry.alloc(entity, tri, tri_uv);
        }
    }
}

pub(super) fn lights(
    mut state: ResMut<ExtractedState>,
    lights: Extract<Query<(&PointLight, &GlobalTransform)>>,
) {
    let state = &mut *state;

    state.lights = Default::default();

    for (point_light, transform) in lights.iter() {
        state.lights.push(st::Light::point(
            transform.translation(),
            color_to_vec3(point_light.color),
            point_light.intensity / 3500.0, // TODO most likely inaccurate
        ));
    }
}

fn color_to_vec3(color: Color) -> Vec3 {
    let [r, g, b, _] = color.as_linear_rgba_f32();

    vec3(r, g, b)
}

pub(super) fn camera(
    mut state: ResMut<ExtractedState>,
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

    state.camera = st::Camera::new(
        transform.translation(),
        transform.translation() + transform.forward(),
        transform.up(),
        size,
        projection.fov,
    );

    state.clear_color = camera_3d.clear_color.clone();
}
