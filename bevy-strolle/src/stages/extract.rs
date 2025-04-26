use std::f32::consts::PI;

use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, CameraRenderGraph};
use bevy::render::sync_world::{RenderEntity, TemporaryRenderEntity};
use bevy::render::view::RenderLayers;
use bevy::render::Extract;
use bevy::utils::HashSet;
use strolle as st;

use crate::graph::StrolleGraph;
use crate::state::{
    ExtractedCamera, ExtractedImage, ExtractedImageData, ExtractedImages,
    ExtractedInstance, ExtractedInstances, ExtractedLight, ExtractedLights,
    ExtractedMaterial, ExtractedMaterials, ExtractedMesh, ExtractedMeshes,
    ExtractedSun,
};
use crate::utils::color_to_vec3;
use crate::{StrolleCamera, StrolleEvent, StrolleSun};

pub(crate) fn meshes(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<Mesh>>>,
    meshes: Extract<Res<Assets<Mesh>>>,
) {
    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed.insert(*id);
            }
            AssetEvent::Removed { id } => {
                removed.push(*id);
            }
            AssetEvent::LoadedWithDependencies { .. }
            | AssetEvent::Unused { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|id| {
        if let Some(mesh) = meshes.get(id) {
            Some(ExtractedMesh {
                handle: id,
                mesh: mesh.to_owned(),
            })
        } else {
            removed.push(id);
            None
        }
    });

    commands.insert_resource(ExtractedMeshes {
        changed: changed.collect(),
        removed,
    });
}

pub(crate) fn materials(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<StandardMaterial>>>,
    materials: Extract<Res<Assets<StandardMaterial>>>,
) {
    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed.insert(*id);
            }
            AssetEvent::Removed { id } => {
                removed.push(*id);
            }
            AssetEvent::LoadedWithDependencies { .. }
            | AssetEvent::Unused { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|id| {
        if let Some(material) = materials.get(id) {
            Some(ExtractedMaterial {
                handle: id,
                material: material.to_owned(),
            })
        } else {
            removed.push(id);
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
    mut dynamic_images: Local<HashSet<AssetId<Image>>>,
) {
    for event in events.read() {
        match event {
            StrolleEvent::MarkImageAsDynamic { id } => {
                dynamic_images.insert(*id);
            }
        }
    }

    // ---

    let mut changed = HashSet::default();
    let mut removed = Vec::new();

    for event in asset_events.read() {
        match event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                changed.insert(*id);
            }
            AssetEvent::Removed { id } => {
                changed.remove(id);
                removed.push(*id);
                dynamic_images.remove(id);
            }
            AssetEvent::LoadedWithDependencies { .. }
            | AssetEvent::Unused { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|id| {
        let Some(image) = images.get(id) else {
            removed.push(id);
            return None;
        };

        let texture_descriptor = image.texture_descriptor.clone();

        let sampler_descriptor = match &image.sampler {
            ImageSampler::Default => wgpu::SamplerDescriptor {
                label: None,
                ..ImageSamplerDescriptor::nearest().as_wgpu()
            },

            ImageSampler::Descriptor(descriptor) => wgpu::SamplerDescriptor {
                label: None,
                ..descriptor.as_wgpu()
            },
        };

        let data = if dynamic_images.contains(&id) {
            let is_legal = image
                .texture_descriptor
                .usage
                .contains(wgpu::TextureUsages::COPY_SRC);

            assert!(
                is_legal,
                "image `{:?}` was marked as dynamic but it is missing the \
                 COPY_SRC usage - please add that usage and try again",
                id
            );

            ExtractedImageData::Texture { is_dynamic: true }
        } else {
            ExtractedImageData::Raw {
                data: image.data.clone(),
            }
        };

        Some(ExtractedImage {
            handle: id,
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
pub(crate) fn instances(
    mut commands: Commands,
    changed: Extract<
        Query<
            (
                Entity,
                &Mesh3d,
                &MeshMaterial3d<StandardMaterial>,
                &GlobalTransform,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Or<(
                Changed<Mesh3d>,
                Changed<MeshMaterial3d<StandardMaterial>>,
                Changed<GlobalTransform>,
                Changed<InheritedVisibility>,
                Changed<RenderLayers>,
            )>,
        >,
    >,
    mut removed: Extract<RemovedComponents<Mesh3d>>,
) {
    let mut removed: Vec<_> = removed.read().collect();

    let changed = changed
        .iter()
        .filter_map(
            |(
                handle,
                mesh_handle,
                material_handle,
                transform,
                visibility,
                layers,
            )| {
                if !visibility.get() {
                    // TODO inefficient; we should push only if the object was
                    //      visible before
                    removed.push(handle);
                    return None;
                }

                // TODO this is invalid (but good enough for now); instead, we
                //      should probably propagate the layers up to the BVH
                //      leaves and adjust the raytracer to read those
                if let Some(layers) = layers {
                    if *layers != RenderLayers::default() {
                        // TODO inefficient; we should push only if the object
                        //      was visible before
                        removed.push(handle);
                        return None;
                    }
                }

                Some(ExtractedInstance {
                    handle,
                    mesh_handle: mesh_handle.id(),
                    material_handle: material_handle.id(),
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
        .read()
        .chain(removed_spot_lights.read())
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
                position: xform.translation(),
                radius: light.radius,
                color: color_to_vec3(light.color) * intensity,
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
                position: translation,
                radius: light.radius,
                color: color_to_vec3(light.color) * intensity,
                range: light.range,
                direction: -(rotation * Vec3::Z).normalize(),
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
            &RenderEntity,
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
        rentity,
        camera,
        camera_rg,
        projection,
        transform,
        strolle_camera,
    ) in cameras.iter()
    {
        if !camera.is_active || !camera_rg.as_dyn_eq().dyn_eq(&StrolleGraph) {
            continue;
        }

        assert!(camera.hdr, "Strolle requires an HDR camera");

        commands
            .entity(rentity.id())
            .insert(ExtractedCamera {
                entity,
                transform: transform.compute_matrix(),
                projection: projection.get_clip_from_view(),
                mode: strolle_camera.map(|camera| camera.mode),
            })
            .insert(TemporaryRenderEntity);
    }
}

pub(crate) fn sun(mut commands: Commands, sun: Extract<Res<StrolleSun>>) {
    commands.insert_resource(ExtractedSun { sun: Some(***sun) });
}
