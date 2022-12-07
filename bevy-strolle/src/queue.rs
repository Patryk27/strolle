use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::{ExtractedView, ViewTarget};
use bevy::utils::HashSet;

use crate::state::ExtractedState;
use crate::StrolleRes;

/// Goes through [`ViewTarget`]s, analyzes their texture formats, and makes sure
/// that we have a renderer for each of them.
///
/// In 99% of the cases there's going to be just one format, `Rgba8UnormSrgb`,
/// but it's possible to get a different one (e.g. when someone activates HDR)
/// or multiple (e.g. when there are two active cameras, one with HDR and the
/// other one - without).
pub(super) fn view(
    device: Res<RenderDevice>,
    strolle: Res<StrolleRes>,
    mut state: ResMut<ExtractedState>,
    view_targets: Query<&ViewTarget, With<ExtractedView>>,
) {
    let device = device.wgpu_device();
    let strolle = &*strolle;

    let alive_texture_formats: HashSet<_> = view_targets
        .iter()
        .map(|vt| vt.main_texture_format())
        .collect();

    for &texture_format in &alive_texture_formats {
        state
            .renderers
            .entry(texture_format)
            .or_insert_with(|| strolle.create_renderer(device, texture_format));
    }

    let dead_texture_formats: Vec<_> = state
        .renderers
        .keys()
        .filter(|tf| !alive_texture_formats.contains(tf))
        .copied()
        .collect();

    for texture_format in dead_texture_formats {
        state.renderers.remove(&texture_format);
    }
}

pub(super) fn submit(
    strolle: Res<StrolleRes>,
    mut state: ResMut<ExtractedState>,
    queue: Res<RenderQueue>,
) {
    state.update(&strolle, &*queue);
}
