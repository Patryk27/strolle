mod extract;
mod prepare;

use bevy::prelude::*;
use bevy::render::{Render, RenderSet};

pub(crate) fn setup(app: &mut SubApp) {
    app.add_systems(
        ExtractSchedule,
        extract::meshes.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::materials.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::instances.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::images.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::lights.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::cameras.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(
        ExtractSchedule,
        extract::sun.in_set(RenderSet::ExtractCommands),
    );

    app.add_systems(Render, prepare::meshes.in_set(RenderSet::Prepare));
    app.add_systems(Render, prepare::materials.in_set(RenderSet::Prepare));

    app.add_systems(
        Render,
        prepare::instances
            .in_set(RenderSet::Prepare)
            .after(prepare::meshes)
            .after(prepare::materials),
    );

    app.add_systems(Render, prepare::images.in_set(RenderSet::Prepare));
    app.add_systems(Render, prepare::lights.in_set(RenderSet::Prepare));
    app.add_systems(Render, prepare::sun.in_set(RenderSet::Prepare));
    app.add_systems(Render, prepare::cameras.in_set(RenderSet::Prepare));

    app.add_systems(
        Render,
        prepare::flush.in_set(RenderSet::PrepareResourcesFlush),
    );
}
