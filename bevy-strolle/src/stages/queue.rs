use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera as BevyExtractedCamera;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewTarget;
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{ExtractedCamera, SyncedCamera, SyncedState};
use crate::utils::GlamCompat;
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
                let Some(size) = bevy_ext_camera.physical_viewport_size else {
                    continue;
                };

                let position = bevy_ext_camera
                    .viewport
                    .as_ref()
                    .map(|v| v.physical_position)
                    .unwrap_or_default();

                st::CameraViewport {
                    format,
                    size: size.compat(),
                    position: position.compat(),
                }
            },

            transform: ext_camera.transform.compat(),
            projection: ext_camera.projection.compat(),
        };

        match state.cameras.entry(entity) {
            Entry::Occupied(entry) => {
                engine.update_camera(device, entry.into_mut().id, camera);
            }

            Entry::Vacant(entry) => {
                entry.insert(SyncedCamera {
                    id: engine.create_camera(device, camera),
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
                engine.delete_camera(camera2.id);
            }

            is_alive
        });
    }
}

pub(crate) fn write(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    mut engine: ResMut<EngineResource>,
    mut state: ResMut<SyncedState>,
) {
    state.tick(&mut engine, &device, &queue);
}
