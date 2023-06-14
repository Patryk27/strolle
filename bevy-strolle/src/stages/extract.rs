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
use crate::utils::{color_to_vec3, GlamCompat};
use crate::{MaterialLike, StrolleCamera};

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

pub(crate) fn materials<M>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    materials: Extract<Res<Assets<M>>>,
) where
    M: MaterialLike,
{
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

pub(crate) fn images(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Image>>>,
    images: Extract<Res<Assets<Image>>>,
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
            if let Some(image) = images.get(&handle) {
                Some((handle, image.to_owned()))
            } else {
                removed.push(handle.clone_weak());
                None
            }
        })
        .collect();

    commands.insert_resource(ExtractedImages { changed, removed });
}

#[allow(clippy::type_complexity)]
pub(crate) fn instances<M>(
    mut commands: Commands,
    all: Extract<Query<Entity, (&Handle<Mesh>, &Handle<M>, &GlobalTransform)>>,
    changed: Extract<
        Query<
            (Entity, &Handle<Mesh>, &Handle<M>, &GlobalTransform),
            Or<(
                Changed<Handle<Mesh>>,
                Changed<Handle<M>>,
                Changed<GlobalTransform>,
            )>,
        >,
    >,
    mut known: Local<HashSet<Entity>>,
) where
    M: MaterialLike,
{
    let changed: Vec<_> = changed
        .iter()
        .map(|(entity, mesh_handle, material_handle, transform)| {
            (
                entity,
                mesh_handle.clone_weak(),
                material_handle.clone_weak(),
                transform.compute_matrix(),
            )
        })
        .collect();

    known.extend(changed.iter().map(|(entity, _, _, _)| entity));

    // ---

    // TODO use `RemovedComponents` instead

    let removed: Vec<_> = known
        .difference(&all.iter().collect::<HashSet<_>>())
        .copied()
        .collect();

    for removed in &removed {
        known.remove(removed);
    }

    // ---

    commands.insert_resource(ExtractedInstances { changed, removed });
}

// TODO use `Changed` to avoid extracting all lights each frame
pub(crate) fn lights(
    mut commands: Commands,
    lights: Extract<Query<(Entity, &PointLight, &GlobalTransform)>>,
) {
    let mut items = Vec::new();

    for (entity, light, transform) in lights.iter() {
        let lum_intensity = light.intensity / (4.0 * PI);

        let light = st::Light::point(
            transform.translation().compat(),
            (color_to_vec3(light.color) * lum_intensity).compat(),
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
            Option<&StrolleCamera>,
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
        strolle_camera,
    ) in cameras.iter()
    {
        if !camera.is_active || **camera_render_graph != crate::graph::NAME {
            continue;
        }

        let Projection::Perspective(projection) = projection else { continue };

        let clear_color = match &camera_3d.clear_color {
            ClearColorConfig::Default => default_clear_color
                .as_ref()
                .map(|cc| cc.0)
                .unwrap_or(Color::BLACK),
            ClearColorConfig::Custom(color) => *color,
            ClearColorConfig::None => {
                // TODO our camera doesn't support transparent clear colors, so
                //      currently this edge case works somewhat differently than
                //      in bevy_render
                Color::rgba(0.0, 0.0, 0.0, 1.0)
            }
        };

        commands.get_or_spawn(entity).insert(ExtractedCamera {
            transform: *transform,
            projection: projection.clone(),
            clear_color,
            mode: strolle_camera.map(|camera| camera.mode),
        });
    }
}
