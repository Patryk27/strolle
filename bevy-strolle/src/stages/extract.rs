use std::collections::VecDeque;
use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, CameraRenderGraph};
use bevy::render::texture::ImageSampler;
use bevy::render::view::RenderLayers;
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
                "image `{:?}` was marked as dynamic but it is missing the \
                 COPY_SRC usage - please add that usage and try again",
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
    children: Extract<Query<(Entity, &Children)>>,
    changed_visibilities: Extract<Query<Entity, Changed<Visibility>>>,
    all: Extract<
        Query<(
            Entity,
            &Handle<Mesh>,
            &Handle<Material>,
            &GlobalTransform,
            &ComputedVisibility,
            Option<&RenderLayers>,
        )>,
    >,
    changed: Extract<
        Query<
            (
                Entity,
                &Handle<Mesh>,
                &Handle<Material>,
                &GlobalTransform,
                &ComputedVisibility,
                Option<&RenderLayers>,
            ),
            Or<(
                Changed<Handle<Mesh>>,
                Changed<Handle<Material>>,
                Changed<GlobalTransform>,
                Changed<RenderLayers>,
            )>,
        >,
    >,
    mut removed: Extract<RemovedComponents<Handle<Mesh>>>,
) where
    Material: MaterialLike,
{
    // TODO switch to `Changed<ComputedVisibility>` after¹ gets fixed
    //      ¹ https://github.com/bevyengine/bevy/issues/8267
    let changed_visibilities = {
        let mut changed = Vec::new();
        let mut pending: VecDeque<_> = changed_visibilities.iter().collect();

        while let Some(entity) = pending.pop_front() {
            if let Ok(payload) = all.get(entity) {
                changed.push(payload);
            }

            if let Ok((_, children)) = children.get(entity) {
                pending.extend(children);
            }
        }

        changed
    };

    // ---

    let mut removed: Vec<_> = removed.iter().collect();

    let changed = changed
        .iter()
        .chain(changed_visibilities)
        .filter_map(
            |(
                handle,
                mesh_handle,
                material_handle,
                transform,
                visibility,
                layers,
            )| {
                if !visibility.is_visible_in_hierarchy() {
                    // TODO inefficient; we should push only if the object was
                    //      visible before
                    removed.push(handle);
                    return None;
                }

                // TODO this is invalid (but good enough for now); instead, we
                //      should probably propagate the layers up to the BVH
                //      leaves and adjust the raytracer to read those
                if let Some(layers) = layers {
                    if *layers != RenderLayers::all() {
                        // TODO inefficient; we should push only if the object
                        //      was visible before
                        removed.push(handle);
                        return None;
                    }
                }

                Some(ExtractedInstance {
                    handle,
                    mesh_handle: mesh_handle.clone_weak(),
                    material_handle: material_handle.clone_weak(),
                    xform: transform.affine(),
                })
            },
        )
        .collect();

    commands.insert_resource(ExtractedInstances { changed, removed });
}

#[allow(clippy::type_complexity)]
pub(crate) fn lights(
    mut commands: Commands,
    changed_point_lights: Extract<
        Query<
            (Entity, &PointLight, &GlobalTransform),
            Or<(Changed<PointLight>, Changed<GlobalTransform>)>,
        >,
    >,
    changed_spot_lights: Extract<
        Query<
            (Entity, &SpotLight, &GlobalTransform),
            Or<(Changed<SpotLight>, Changed<GlobalTransform>)>,
        >,
    >,
    mut removed_point_lights: Extract<RemovedComponents<PointLight>>,
    mut removed_spot_lights: Extract<RemovedComponents<SpotLight>>,
) {
    let mut removed: Vec<_> = removed_point_lights
        .iter()
        .chain(removed_spot_lights.iter())
        .collect();

    let changed_point_lights: Vec<_> = changed_point_lights
        .iter()
        .filter_map(|(handle, light, xform)| {
            let intensity = light.intensity / (4.0 * PI);

            if intensity < 0.0001 {
                removed.push(handle);
                return None;
            }

            let light = st::Light::Point {
                position: xform.translation().compat(),
                radius: light.radius,
                color: (color_to_vec3(light.color) * intensity).compat(),
                range: light.range,
            };

            Some(ExtractedLight { handle, light })
        })
        .collect();

    let changed_spot_lights: Vec<_> = changed_spot_lights
        .iter()
        .filter_map(|(handle, light, xform)| {
            let intensity = light.intensity / (4.0 * PI);

            if intensity < 0.0001 {
                removed.push(handle);
                return None;
            }

            let (_, rotation, translation) =
                xform.to_scale_rotation_translation();

            let light = st::Light::Spot {
                position: translation.compat(),
                radius: light.radius,
                color: (color_to_vec3(light.color) * intensity).compat(),
                range: light.range,
                direction: -(rotation * Vec3::Z).normalize().compat(),
                angle: light.outer_angle,
            };

            Some(ExtractedLight { handle, light })
        })
        .collect();

    let changed = changed_point_lights
        .into_iter()
        .chain(changed_spot_lights)
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

        assert!(camera.hdr, "Strolle requires an HDR camera");

        commands.get_or_spawn(entity).insert(ExtractedCamera {
            transform: transform.compute_matrix(),
            projection: projection.get_projection_matrix(),
            mode: strolle_camera.map(|camera| camera.mode),
        });
    }
}

pub(crate) fn sun(mut commands: Commands, sun: Extract<Res<StrolleSun>>) {
    commands.insert_resource(ExtractedSun { sun: Some(***sun) });
}
