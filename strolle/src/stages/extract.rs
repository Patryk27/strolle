use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor};
use bevy::render::view::RenderLayers;
use bevy::render::Extract;
use bevy::utils::HashSet;

use crate::camera::ExtractedStrolleCamera;
use crate::utils::color_to_vec3;
use crate::{
    Event, ExtractedImage, ExtractedImageData, ExtractedImages,
    ExtractedInstance, ExtractedInstances, ExtractedLight, ExtractedLights,
    ExtractedMaterial, ExtractedMaterials, ExtractedMesh, ExtractedMeshes,
    ExtractedSun, ImageHandle, InstanceHandle, Light, LightHandle,
    MaterialHandle, MeshHandle, Sun,
};

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
                removed.push(MeshHandle::new(*id));
            }
            AssetEvent::LoadedWithDependencies { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| {
        if let Some(mesh) = meshes.get(handle) {
            Some(ExtractedMesh {
                handle: MeshHandle::new(handle),
                mesh: mesh.to_owned(),
            })
        } else {
            removed.push(MeshHandle::new(handle));
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
                removed.push(MaterialHandle::new(*id));
            }
            AssetEvent::LoadedWithDependencies { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| {
        if let Some(material) = materials.get(handle) {
            Some(ExtractedMaterial {
                handle: MaterialHandle::new(handle),
                material: material.to_owned(),
            })
        } else {
            removed.push(MaterialHandle::new(handle));
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
    mut events: Extract<EventReader<Event>>,
    mut asset_events: Extract<EventReader<AssetEvent<Image>>>,
    images: Extract<Res<Assets<Image>>>,
    mut dynamic_images: Local<HashSet<AssetId<Image>>>,
) {
    for event in events.read() {
        match event {
            Event::MarkImageAsDynamic { id } => {
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
                removed.push(ImageHandle::new(*id));
                dynamic_images.remove(id);
            }
            AssetEvent::LoadedWithDependencies { .. } => {
                //
            }
        }
    }

    let changed = changed.into_iter().flat_map(|handle| {
        let Some(image) = images.get(handle) else {
            removed.push(ImageHandle::new(handle));
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
            handle: ImageHandle::new(handle),
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
                &Handle<Mesh>,
                &Handle<StandardMaterial>,
                &GlobalTransform,
                &InheritedVisibility,
                Option<&RenderLayers>,
            ),
            Or<(
                Changed<Handle<Mesh>>,
                Changed<Handle<StandardMaterial>>,
                Changed<GlobalTransform>,
                Changed<InheritedVisibility>,
                Changed<RenderLayers>,
            )>,
        >,
    >,
    mut removed: Extract<RemovedComponents<Handle<Mesh>>>,
) {
    let mut removed: Vec<_> = removed.read().map(InstanceHandle::new).collect();

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
                let handle = InstanceHandle::new(handle);

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
                    if *layers != RenderLayers::all() {
                        // TODO inefficient; we should push only if the object
                        //      was visible before
                        removed.push(handle);
                        return None;
                    }
                }

                Some(ExtractedInstance {
                    handle,
                    mesh_handle: MeshHandle::new(mesh_handle.id()),
                    material_handle: MaterialHandle::new(material_handle.id()),
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
        .map(LightHandle::new)
        .collect();

    let changed_point_lights: Vec<_> = changed_point_lights
        .iter()
        .filter_map(|(handle, light, xform)| {
            let handle = LightHandle::new(handle);
            let intensity = light.intensity / (4.0 * PI);

            if intensity < 0.0001 {
                // TODO inefficient; we should push only if the light was
                //      visible before
                removed.push(handle);
                return None;
            }

            let light = Light::Point {
                position: xform.translation(),
                radius: light.radius,
                color: (color_to_vec3(light.color) * intensity),
                range: light.range,
            };

            Some(ExtractedLight { handle, light })
        })
        .collect();

    let changed_spot_lights: Vec<_> = changed_spot_lights
        .iter()
        .filter_map(|(handle, light, xform)| {
            let handle = LightHandle::new(handle);
            let intensity = light.intensity / (4.0 * PI);

            if intensity < 0.0001 {
                removed.push(handle);
                return None;
            }

            let (_, rotation, translation) =
                xform.to_scale_rotation_translation();

            let light = Light::Spot {
                position: translation,
                radius: light.radius,
                color: (color_to_vec3(light.color) * intensity),
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
    cameras: Extract<Query<(Entity, &Camera, &CameraRenderGraph)>>,
) {
    for (entity, camera, camera_render_graph) in cameras.iter() {
        if !camera.is_active
            || **camera_render_graph != crate::graph::BVH_HEATMAP
        {
            continue;
        }

        assert!(camera.hdr, "Strolle requires an HDR camera");

        commands.get_or_spawn(entity).insert(ExtractedStrolleCamera);
    }
}

pub(crate) fn sun(mut commands: Commands, sun: Extract<Res<Sun>>) {
    commands.insert_resource(ExtractedSun { sun: Some(**sun) });
}
