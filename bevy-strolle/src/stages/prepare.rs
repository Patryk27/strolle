use std::mem;

use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera as BevyExtractedCamera;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_asset::{RenderAssetUsages, RenderAssets};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewTarget;
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::HashSet;
use strolle as st;
use strolle::{ImageData, ToGpu};

use crate::state::{
    ExtractedCamera, ExtractedImageData, ExtractedImages, ExtractedInstances,
    ExtractedLights, ExtractedMaterials, ExtractedMeshes, ExtractedSun,
    SyncedCamera, SyncedState,
};
use crate::utils::color_to_vec4;
use crate::{EngineParams, EngineResource};

pub fn meshes(
    mut engine: ResMut<EngineResource>,
    mut meshes: ResMut<ExtractedMeshes>,
) {
    for handle in meshes
        .removed
        .iter()
        .copied()
        .chain(meshes.changed.iter().map(|mesh| mesh.handle))
    {
        engine.remove_mesh(handle);
    }

    for mesh in mem::take(&mut meshes.changed) {
        if mesh.mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            continue;
        }

        let mesh_positions = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no positions", mesh.handle);
            });

        let mesh_normals = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no normals", mesh.handle);
            });

        let mesh_uvs = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x2(uvs) => uvs,
                _ => {
                    panic!(
                        "mesh {:?} uses unsupported format for UVs",
                        mesh.handle
                    )
                }
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_tans = mesh
            .mesh
            .attribute(Mesh::ATTRIBUTE_TANGENT)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x4(tangents) => tangents,
                _ => panic!(
                    "mesh {:?} uses unsupported format for tangents",
                    mesh.handle
                ),
            })
            .map(|tangents| tangents.as_slice())
            .unwrap_or(&[]);

        let mesh_indices: Vec<_> = mesh
            .mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no indices", mesh.handle);
            })
            .iter()
            .collect();

        let mesh_triangles: Vec<_> = mesh_indices
            .chunks(3)
            .map(|vs| {
                let position0 = mesh_positions[vs[0]];
                let position1 = mesh_positions[vs[1]];
                let position2 = mesh_positions[vs[2]];

                let normal0 = mesh_normals[vs[0]];
                let normal1 = mesh_normals[vs[1]];
                let normal2 = mesh_normals[vs[2]];

                let uv0 = mesh_uvs.get(vs[0]).copied().unwrap_or_default();
                let uv1 = mesh_uvs.get(vs[1]).copied().unwrap_or_default();
                let uv2 = mesh_uvs.get(vs[2]).copied().unwrap_or_default();

                let tan0 = mesh_tans.get(vs[0]).copied().unwrap_or_default();
                let tan1 = mesh_tans.get(vs[1]).copied().unwrap_or_default();
                let tan2 = mesh_tans.get(vs[2]).copied().unwrap_or_default();

                st::MeshTriangle::default()
                    .with_positions([
                        Vec3::from(position0),
                        Vec3::from(position1),
                        Vec3::from(position2),
                    ])
                    .with_normals([
                        Vec3::from(normal0),
                        Vec3::from(normal1),
                        Vec3::from(normal2),
                    ])
                    .with_uvs([
                        Vec2::from(uv0),
                        Vec2::from(uv1),
                        Vec2::from(uv2),
                    ])
                    .with_tangents([
                        Vec4::from(tan0),
                        Vec4::from(tan1),
                        Vec4::from(tan2),
                    ])
            })
            .collect();

        engine.insert_mesh(mesh.handle, st::Mesh::new(mesh_triangles));
    }
}

pub fn materials(
    mut engine: ResMut<EngineResource>,
    mut materials: ResMut<ExtractedMaterials>,
) {
    for handle in &materials.removed {
        engine.remove_material(*handle);
    }

    let map = |mat: StandardMaterial| {
        let base_color = {
            let color = color_to_vec4(mat.base_color);

            match mat.alpha_mode {
                AlphaMode::Opaque => color.xyz().extend(1.0),
                AlphaMode::Mask(mask) => {
                    if color.w >= mask {
                        color.xyz().extend(1.0)
                    } else {
                        color.xyz().extend(0.0)
                    }
                }
                _ => color,
            }
        };

        let ior = if mat.thickness > 0.0 { mat.ior } else { 1.0 };

        let alpha_mode = match mat.alpha_mode {
            AlphaMode::Opaque => st::AlphaMode::Opaque,
            _ => st::AlphaMode::Blend,
        };

        st::Material {
            base_color: base_color.to_gpu(),
            base_color_texture: mat
                .base_color_texture
                .map(|handle| handle.id()),
            emissive: color_to_vec4(Color::LinearRgba(mat.emissive)).to_gpu(),
            emissive_texture: mat.emissive_texture.map(|handle| handle.id()),
            perceptual_roughness: mat.perceptual_roughness,
            metallic: mat.metallic,
            metallic_roughness_texture: mat
                .metallic_roughness_texture
                .map(|handle| handle.id()),
            reflectance: mat.reflectance,
            normal_map_texture: mat
                .normal_map_texture
                .map(|handle| handle.id()),
            ior,
            alpha_mode,
        }
    };

    for entry in materials.changed.drain(..) {
        engine.insert_material(entry.handle, map(entry.material));
    }
}

pub fn images(
    mut engine: ResMut<EngineResource>,
    textures: Res<RenderAssets<GpuImage>>,
    mut images: ResMut<ExtractedImages>,
) {
    for handle in &images.removed {
        engine.remove_image(*handle);
    }

    for entry in mem::take(&mut images.changed) {
        if entry.texture_descriptor.dimension != wgpu::TextureDimension::D2 {
            continue;
        }

        let data = match entry.data {
            ExtractedImageData::Raw { data } => st::ImageData::Raw { data },

            ExtractedImageData::Texture { is_dynamic } => {
                let Some(gpu_image) = textures.get(entry.handle) else {
                    warn!("Missing GPU image for handle {:?}", entry.handle);
                    continue;
                };

                st::ImageData::Texture {
                    texture: gpu_image.texture.clone(),
                    is_dynamic,
                }
            }
        };

        // TODO we should add only those images which are used by at least one
        //      material, since otherwise we'll .add_image() textures that are
        //      related solely to UI, for instance
        //
        //      (conversely, we should remove those images which are not in use
        //      by any material)
        //
        //      that's not so easy though because it can happen that an image is
        //      loaded first *and then* (e.g. in next frame) it's used by some
        //      material, in which case a simple condition right here will not
        //      be sufficient
        engine.insert_image(
            entry.handle,
            st::Image::new(
                data,
                entry.texture_descriptor,
                entry.sampler_descriptor,
            ),
        );
    }
}

pub fn instances(
    mut engine: ResMut<EngineResource>,
    mut instances: ResMut<ExtractedInstances>,
) {
    for handle in &instances.removed {
        engine.remove_instance(*handle);
    }

    for entry in mem::take(&mut instances.changed) {
        engine.insert_instance(
            entry.handle,
            st::Instance::new(
                entry.mesh_handle,
                entry.material_handle,
                entry.xform,
            ),
        );
    }
}

pub fn lights(
    mut engine: ResMut<EngineResource>,
    mut lights: ResMut<ExtractedLights>,
) {
    for handle in &lights.removed {
        engine.remove_light(*handle);
    }

    for entry in mem::take(&mut lights.changed) {
        engine.insert_light(entry.handle, entry.light);
    }
}

pub fn sun(mut engine: ResMut<EngineResource>, mut sun: ResMut<ExtractedSun>) {
    if let Some(sun) = sun.sun.take() {
        engine.update_sun(sun);
    }
}

pub fn cameras(
    device: Res<RenderDevice>,
    mut state: ResMut<SyncedState>,
    mut engine: ResMut<EngineResource>,
    mut cameras: Query<(
        Entity,
        &ViewTarget,
        &BevyExtractedCamera,
        &ExtractedCamera,
    )>,
) {
    let device = device.wgpu_device();
    let state = &mut *state;
    let engine = &mut *engine;
    let mut alive_cameras = HashSet::new();

    for (entity, view_target, bevy_ext_camera, ext_camera) in cameras.iter_mut()
    {
        let format = view_target.main_texture_format();

        let Some(size) = bevy_ext_camera.physical_viewport_size else {
            continue;
        };

        let position = bevy_ext_camera
            .viewport
            .as_ref()
            .map(|v| v.physical_position)
            .unwrap_or_default();

        let camera = st::Camera {
            mode: ext_camera.mode.unwrap_or_default(),
            viewport: {
                st::CameraViewport {
                    format,
                    size,
                    position,
                }
            },
            transform: ext_camera.transform,
            projection: ext_camera.projection,
        };

        match state.cameras.entry(entity) {
            Entry::Occupied(entry) => {
                engine.update_camera(device, entry.into_mut().handle, camera);
            }

            Entry::Vacant(entry) => {
                entry.insert(SyncedCamera {
                    handle: engine.create_camera(device, camera),
                });
            }
        }

        alive_cameras.insert(entity);
    }

    // -----

    if alive_cameras.len() != state.cameras.len() {
        state.cameras.retain(|entity2, camera2| {
            let is_alive = alive_cameras.contains(entity2);

            if !is_alive {
                engine.delete_camera(camera2.handle);
            }

            is_alive
        });
    }
}

pub(crate) fn flush(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut engine: ResMut<EngineResource>,
    mut state: ResMut<SyncedState>,
) {
    state.tick(&mut engine, &device, &queue);
}
