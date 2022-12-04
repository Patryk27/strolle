use std::f32::consts::PI;

use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::utils::HashSet;
use strolle as st;

use crate::State;

pub(super) fn geometry(
    mut state: ResMut<State>,
    meshes: Extract<Res<Assets<Mesh>>>,
    models: Extract<
        Query<(Entity, &Handle<Mesh>, &Transform), Added<Handle<Mesh>>>,
    >,
) {
    let state = &mut *state;

    for (entity, mesh, &transform) in models.iter() {
        let mesh = meshes.get(mesh).unwrap();
        let transform = transform.compute_matrix();

        state.geometry.builder().add(entity, mesh, transform);
    }
}

// TODO: We also need to sync mesh data with appropriate assigned materials
pub(super) fn materials(
    mut state: ResMut<State>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
    material_instances: Extract<Query<&Handle<StandardMaterial>>>,
) {
    let state = &mut *state;

    state.materials = Default::default();

    let mut unique_materials = HashSet::new();

    for material in material_instances.iter() {
        unique_materials.insert(material);
    }

    for (idx, material) in unique_materials.into_iter().enumerate() {
        let material = materials.get(material).unwrap();

        state.materials.set(
            st::MaterialId::new(idx),
            st::Material::none().with_color(color_to_vec3(material.base_color)),
        );
    }
}

pub(super) fn lights(
    mut state: ResMut<State>,
    lights: Extract<Query<(&PointLight, &GlobalTransform)>>,
) {
    let state = &mut *state;

    state.lights = Default::default();

    for (point_light, transform) in lights.iter() {
        state.lights.push(st::Light::point(
            transform.translation(),
            color_to_vec3(point_light.color),
            point_light.intensity,
        ));
    }
}

fn color_to_vec3(color: Color) -> Vec3 {
    vec3(color.r(), color.g(), color.b())
}

pub(super) fn camera(
    mut state: ResMut<State>,
    cameras: Extract<Query<(&Camera, &GlobalTransform)>>,
) {
    for (camera, transform) in cameras.iter() {
        if !camera.is_active {
            continue;
        }

        state.camera = st::Camera::new(
            transform.translation(),
            transform.translation() + transform.forward(),
            transform.up(),
            1.0,
            vec2(256.0, 256.0),
            PI / 2.0,
        );
    }
}
