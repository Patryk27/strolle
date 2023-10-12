mod camera;
mod event;
mod material;
mod render_node;
mod stages;
mod state;
mod sun;
mod utils;

pub mod prelude {
    pub use crate::*;
}

pub mod graph {
    pub const NAME: &str = "strolle";

    pub mod node {
        pub const RENDER: &str = "strolle_render";
        pub const TONEMAPPING: &str = "strolle_tonemapping";
        pub const UPSCALING: &str = "strolle_upscaling";
    }
}

use std::ops;

use bevy::core_pipeline::tonemapping::TonemappingNode;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::Texture;
use bevy::render::renderer::RenderDevice;
use bevy::render::{Render, RenderApp, RenderSet};
pub use strolle as st;

pub use self::camera::*;
pub use self::event::*;
pub use self::material::*;
use self::render_node::*;
use self::state::*;
pub use self::sun::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StrolleEvent>();
        app.add_asset::<StrolleMaterial>();
        app.insert_resource(StrolleSun::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(SyncedState::default());

        // -------------------------- //
        // RenderSet::ExtractCommands //

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::meshes.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::materials::<StandardMaterial>
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::materials::<StrolleMaterial>
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::instances::<StandardMaterial>
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::instances::<StrolleMaterial>
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::images.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::lights.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::cameras.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::sun.in_set(RenderSet::ExtractCommands),
        );

        // ------------------ //
        // RenderSet::Prepare //

        render_app.add_systems(
            Render,
            stages::prepare::meshes.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::materials::<StandardMaterial>
                .in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::materials::<StrolleMaterial>
                .in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::instances::<StandardMaterial>
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials::<StandardMaterial>),
        );

        render_app.add_systems(
            Render,
            stages::prepare::instances::<StrolleMaterial>
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials::<StrolleMaterial>),
        );

        render_app.add_systems(
            Render,
            stages::prepare::images.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::lights.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::sun.in_set(RenderSet::Prepare),
        );

        // ---------------- //
        // RenderSet::Queue //

        render_app.add_systems(
            Render,
            stages::queue::cameras.in_set(RenderSet::Queue),
        );

        render_app.add_systems(
            Render,
            stages::queue::write
                .in_set(RenderSet::Queue)
                .after(stages::queue::cameras),
        );

        // -----

        render_app
            .add_render_sub_graph(graph::NAME)
            .add_render_graph_node::<ViewNodeRunner<RenderNode>>(
                graph::NAME,
                graph::node::RENDER,
            )
            .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(
                graph::NAME,
                graph::node::TONEMAPPING,
            )
            .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
                graph::NAME,
                graph::node::UPSCALING,
            )
            .add_render_graph_edges(
                graph::NAME,
                &[
                    graph::node::RENDER,
                    graph::node::TONEMAPPING,
                    graph::node::UPSCALING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>();
        let engine = st::Engine::new(render_device.wgpu_device());

        render_app.insert_resource(EngineResource(engine));
    }
}

#[derive(Resource)]
struct EngineResource(st::Engine<EngineParams>);

#[derive(Clone, Debug)]
struct EngineParams;

impl st::Params for EngineParams {
    type ImageHandle = Handle<Image>;
    type ImageTexture = Texture;
    type InstanceHandle = Entity;
    type LightHandle = Entity;
    type MaterialHandle = MaterialHandle;
    type MeshHandle = Handle<Mesh>;
}

impl ops::Deref for EngineResource {
    type Target = st::Engine<EngineParams>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for EngineResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
