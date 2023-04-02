use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera as BevyExtractedCamera;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewTarget;
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{ExtractedCamera, SyncedCamera, SyncedState};
use crate::utils::color_to_vec3;
use crate::EngineResource;

pub(crate) fn cameras(
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
        let camera = st::Camera {
            mode: ext_camera.mode.unwrap_or_default(),

            viewport: {
                let format = view_target.main_texture_format();

                let Some(size) = bevy_ext_camera.physical_viewport_size else { continue };

                let position = bevy_ext_camera
                    .viewport
                    .as_ref()
                    .map(|v| v.physical_position)
                    .unwrap_or_default();

                st::CameraViewport {
                    format,
                    size,
                    position,
                }
            },

            projection: {
                let projection_view =
                    Mat4::perspective_rh(
                        ext_camera.projection.fov,
                        ext_camera.projection.aspect_ratio,
                        ext_camera.projection.near,
                        ext_camera.projection.far,
                    ) * ext_camera.transform.compute_matrix().inverse();

                let origin = ext_camera.transform.translation();

                let look_at = ext_camera.transform.translation()
                    + ext_camera.transform.forward();

                let up = ext_camera.transform.up();
                let fov = ext_camera.projection.fov;

                st::CameraProjection {
                    projection_view,
                    origin,
                    look_at,
                    up,
                    fov,
                }
            },

            background: st::CameraBackground {
                color: color_to_vec3(ext_camera.clear_color),
            },
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
        state.cameras.drain_filter(|entity2, camera2| {
            let is_dead = !alive_cameras.contains(entity2);

            if is_dead {
                engine.delete_camera(camera2.handle);
            }

            is_dead
        });
    }
}

pub(crate) fn write(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut engine: ResMut<EngineResource>,
    mut state: ResMut<SyncedState>,
) {
    state.flush(&mut engine, &device, &queue);
}
