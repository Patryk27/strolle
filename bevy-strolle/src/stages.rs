mod extract;
mod prepare;

use bevy::prelude::*;
use bevy::render::{Render, RenderSet};

pub(crate) fn setup(render_app: &mut App) {
    render_app.add_systems(
        ExtractSchedule,
        extract::meshes.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::materials.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::instances.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::images.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::lights.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::cameras.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(
        ExtractSchedule,
        extract::sun.in_set(RenderSet::ExtractCommands),
    );

    render_app.add_systems(Render, prepare::meshes.in_set(RenderSet::Prepare));

    render_app
        .add_systems(Render, prepare::materials.in_set(RenderSet::Prepare));

    render_app.add_systems(
        Render,
        prepare::instances
            .in_set(RenderSet::Prepare)
            .after(prepare::meshes)
            .after(prepare::materials),
    );

    render_app.add_systems(Render, prepare::images.in_set(RenderSet::Prepare));
    render_app.add_systems(Render, prepare::lights.in_set(RenderSet::Prepare));
    render_app.add_systems(Render, prepare::sun.in_set(RenderSet::Prepare));
    render_app.add_systems(Render, prepare::cameras.in_set(RenderSet::Prepare));

    render_app
        .add_systems(Render, prepare::flush.in_set(RenderSet::PrepareFlush));
}
