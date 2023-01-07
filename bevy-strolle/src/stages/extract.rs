use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::Extract;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{
    ExtractedCamera, ExtractedImages, ExtractedInstances, ExtractedLights,
    ExtractedMaterials, ExtractedMeshes,
};
use crate::utils::color_to_vec3;

pub(crate) fn meshes(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Mesh>>>,
    meshes: Extract<Res<Assets<Mesh>>>,
) {
    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in events.iter() {
        match event {
            AssetEvent::Created { handle }
            | AssetEvent::Modified { handle } => {
                changed.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let changed = changed
        .into_iter()
        .flat_map(|handle| {
            if let Some(mesh) = meshes.get(&handle) {
                Some((handle, mesh.to_owned()))
            } else {
                removed.push(handle.clone_weak());
                None
            }
        })
        .collect();

    commands.insert_resource(ExtractedMeshes { changed, removed });
}

pub(crate) fn images(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Image>>>,
) {
    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in events.iter() {
        match event {
            AssetEvent::Created { handle }
            | AssetEvent::Modified { handle } => {
                changed.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    // Usually we'd map `Handle<Image>` into `GpuImage` right here (similarly as
    // we do with meshes), but `RenderAssets<Image>` gets filled out during the
    // *prepare* phase, so we can't read it just yet.
    //
    // So instead we're just passing `Handle<Image>` that gets converted into an
    // actual image later.

    commands.insert_resource(ExtractedImages { changed, removed });
}

pub(crate) fn materials(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<StandardMaterial>>>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
) {
    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in events.iter() {
        match event {
            AssetEvent::Created { handle }
            | AssetEvent::Modified { handle } => {
                changed.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let changed = changed
        .into_iter()
        .flat_map(|handle| {
            if let Some(material) = materials.get(&handle) {
                Some((handle, material.to_owned()))
            } else {
                removed.push(handle.clone_weak());
                None
            }
        })
        .collect();

    commands.insert_resource(ExtractedMaterials { changed, removed });
}

#[allow(clippy::type_complexity)]
pub(crate) fn instances(
    mut commands: Commands,
    instances: Extract<
        Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform)>,
    >,
) {
    let mut items = Vec::new();

    for (mesh_handle, material_handle, transform) in instances.iter() {
        items.push((
            mesh_handle.clone_weak(),
            material_handle.clone_weak(),
            transform.compute_matrix(),
        ));
    }

    commands.insert_resource(ExtractedInstances { items });
}

pub(crate) fn lights(
    mut commands: Commands,
    lights: Extract<Query<(Entity, &PointLight, &GlobalTransform)>>,
) {
    let mut items = Vec::new();

    for (entity, light, transform) in lights.iter() {
        let lum_intensity = light.intensity / (4.0 * PI);

        let light = st::Light::point(
            transform.translation(),
            color_to_vec3(light.color) * lum_intensity,
            light.range,
        );

        items.push((entity, light));
    }

    commands.insert_resource(ExtractedLights { items });
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
