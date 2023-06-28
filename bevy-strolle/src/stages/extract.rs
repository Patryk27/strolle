use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::texture::ImageSampler;
use bevy::render::Extract;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{
    ExtractedCamera, ExtractedImage, ExtractedImageData, ExtractedImages,
    ExtractedInstance, ExtractedInstances, ExtractedLight, ExtractedLights,
    ExtractedMaterial, ExtractedMaterials, ExtractedMesh, ExtractedMeshes,
    ExtractedSun,
};
use crate::utils::{color_to_vec3, GlamCompat};
use crate::{MaterialLike, StrolleCamera, StrolleEvent, StrolleSun};

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
                removed.push(handle.clone_weak());
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| {
        if let Some(mesh) = meshes.get(&handle) {
            Some(ExtractedMesh {
                handle,
                mesh: mesh.to_owned(),
            })
        } else {
            removed.push(handle.clone_weak());
            None
        }
    });

    commands.insert_resource(ExtractedMeshes {
        changed: changed.collect(),
        removed,
    });
}

pub(crate) fn materials<Material>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Material>>>,
    materials: Extract<Res<Assets<Material>>>,
) where
    Material: MaterialLike,
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
                removed.push(handle.clone_weak());
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| {
        if let Some(material) = materials.get(&handle) {
            Some(ExtractedMaterial {
                handle,
                material: material.to_owned(),
            })
        } else {
            removed.push(handle.clone_weak());
            None
        }
    });

    commands.insert_resource(ExtractedMaterials {
        changed: changed.collect(),
        removed,
    });
}

pub(crate) fn images(
    mut commands: Commands,
    mut events: Extract<EventReader<StrolleEvent>>,
    mut asset_events: Extract<EventReader<AssetEvent<Image>>>,
    images: Extract<Res<Assets<Image>>>,
    mut dynamic_images: Local<HashSet<Handle<Image>>>,
) {
    for event in events.iter() {
        match event {
            StrolleEvent::MarkImageAsDynamic { handle } => {
                dynamic_images.insert(handle.clone_weak());
            }
        }
    }

    // ---

    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in asset_events.iter() {
        match event {
            AssetEvent::Created { handle }
            | AssetEvent::Modified { handle } => {
                changed.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed.remove(handle);
                removed.push(handle.clone_weak());
                dynamic_images.remove(handle);
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| -> Option<_> {
        let Some(image) = images.get(&handle) else {
            removed.push(handle);
            return None;
        };

        let texture_descriptor = image.texture_descriptor.clone();

        let sampler_descriptor = match &image.sampler_descriptor {
            ImageSampler::Default => {
                // According to Bevy's docs, this should read the defaults as
                // specified in the `ImagePlugin`'s setup, but it seems that it
                // is not actually possible for us to access that value in here.
                //
                // So let's to the next best thing: assume our own default!
                ImageSampler::nearest_descriptor()
            }

            ImageSampler::Descriptor(descriptor) => descriptor.clone(),
        };

        let data = if dynamic_images.contains(&handle) {
            let is_legal = image
                .texture_descriptor
                .usage
                .contains(wgpu::TextureUsages::COPY_SRC);

            assert!(
                is_legal,
                "Image `{:?}` was marked as dynamic but it is missing the \
                 COPY_SRC usage; please add that usage and try again",
                handle
            );

            ExtractedImageData::Texture { is_dynamic: true }
        } else {
            ExtractedImageData::Raw {
                data: image.data.clone(),
            }
        };

        Some(ExtractedImage {
            handle,
            texture_descriptor,
            sampler_descriptor,
            data,
        })
    });

    commands.insert_resource(ExtractedImages {
        changed: changed.collect(),
        removed,
    });
}

#[allow(clippy::type_complexity)]
pub(crate) fn instances<Material>(
    mut commands: Commands,
    changed: Extract<
        Query<
            (Entity, &Handle<Mesh>, &Handle<Material>, &GlobalTransform),
            Or<(
                Changed<Handle<Mesh>>,
                Changed<Handle<Material>>,
                Changed<GlobalTransform>,
            )>,
        >,
    >,
    mut removed: Extract<RemovedComponents<Handle<Mesh>>>,
) where
    Material: MaterialLike,
{
    let changed = changed
        .iter()
        .map(|(handle, mesh_handle, material_handle, transform)| {
            ExtractedInstance {
                handle,
                mesh_handle: mesh_handle.clone_weak(),
                material_handle: material_handle.clone_weak(),
                xform: transform.affine(),
            }
        })
        .collect();

    let removed = removed
        .iter()
        .map(|removed| removed.clone().into())
        .collect();

    commands.insert_resource(ExtractedInstances { changed, removed });
}

pub(crate) fn lights(
    mut commands: Commands,
    changed: Extract<
        Query<
            (Entity, &PointLight, &GlobalTransform),
            Or<(Changed<PointLight>, Changed<GlobalTransform>)>,
        >,
    >,
    mut removed: Extract<RemovedComponents<PointLight>>,
) {
    let changed = changed
        .iter()
        .map(|(handle, light, xform)| {
            let lum_intensity = light.intensity / (4.0 * PI);

            let light = st::Light::point(
                xform.translation().compat(),
                light.radius,
                (color_to_vec3(light.color) * lum_intensity).compat(),
                light.range,
            );

            ExtractedLight { handle, light }
        })
        .collect();

    let removed = removed
        .iter()
        .map(|removed| removed.clone().into())
        .collect();

    commands.insert_resource(ExtractedLights { changed, removed });
}

#[allow(clippy::type_complexity)]
pub(crate) fn cameras(
    mut commands: Commands,
    cameras: Extract<
        Query<(
            Entity,
            &Camera,
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

        commands.get_or_spawn(entity).insert(ExtractedCamera {
            transform: *transform,
            projection: projection.clone(),
            mode: strolle_camera.map(|camera| camera.mode),
        });
    }
}

pub(crate) fn sun(mut commands: Commands, sun: Extract<Res<StrolleSun>>) {
    commands.insert_resource(ExtractedSun {
        sun: Some((***sun).clone()),
    });
}
