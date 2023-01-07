use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera as BevyExtractedCamera;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewTarget;
use bevy::utils::HashSet;
use strolle as st;

use crate::state::{ExtractedCamera, SyncedState, SyncedView};
use crate::utils::color_to_vec3;
use crate::EngineResource;

pub(crate) fn viewports(
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
    let mut alive_views = HashSet::new();

    for (entity, view_target, bevy_ext_camera, ext_camera) in cameras.iter_mut()
    {
        let texture_format = view_target.main_texture_format();

        let viewport_pos = bevy_ext_camera
            .viewport
            .as_ref()
            .map(|v| v.physical_position)
            .unwrap_or_default();

        let Some(viewport_size) = bevy_ext_camera.physical_viewport_size else { continue };

        if let Some(view) = state.views.get(&entity) {
            let mut invalidated = false;

            if view_target.main_texture_format() != view.viewport.format() {
                log::debug!(
                    "Camera {:?} invalidated: texture format has been changed",
                    entity,
                );

                // This can happen e.g. if the camera's being switched into HDR
                // mode - if that happens, we have to re-generate the viewport,
                // since otherwise rendering to it will fail due to texture
                // format mismatch.
                //
                // Note that we don't really care if the camera's in HDR mode or
                // not - the only important thing here is that the texture
                // format has changed and we have to adapt.
                invalidated = true;
            }

            if viewport_pos != view.viewport.pos()
                || viewport_size != view.viewport.size()
            {
                log::debug!(
                    "Camera {:?} invalidated: viewport has been changed",
                    entity
                );

                // This can happen if the camera's being resized.
                //
                // TODO invalidating the viewport is a pretty expensive
                //      operation - it would be nice if we had some kind or
                //      `viewport.resize()` function that would work in-place
                invalidated = true;
            }

            if invalidated {
                state.views.remove(&entity);
            }
        }

        let camera = st::Camera::new(
            ext_camera.transform.translation(),
            ext_camera.transform.translation() + ext_camera.transform.forward(),
            ext_camera.transform.up(),
            viewport_pos,
            viewport_size,
            ext_camera.projection.fov,
            color_to_vec3(ext_camera.clear_color),
        );

        if let Some(view) = state.views.get_mut(&entity) {
            view.viewport.set_camera(camera);
        } else {
            log::debug!("Camera {:?} extracted", entity);

            let viewport = engine.create_viewport(
                device,
                viewport_pos,
                viewport_size,
                texture_format,
                camera,
            );

            state.views.insert(entity, SyncedView { viewport });
        }

        alive_views.insert(entity);
    }

    // -----

    if alive_views.len() != state.views.len() {
        state.views.drain_filter(|entity2, _| {
            let is_dead = !alive_views.contains(entity2);

            if is_dead {
                log::debug!("Camera {:?} died", entity2);
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
    state.write(&mut engine, &device, &queue);
}
