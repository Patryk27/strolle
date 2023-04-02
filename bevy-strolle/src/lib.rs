mod camera;
mod material;
mod render_node;
mod stages;
mod state;
mod utils;

pub mod prelude {
    pub use crate::*;
}

pub mod graph {
    pub const NAME: &str = "strolle";

    pub mod node {
        pub const RENDER: &str = "strolle_render";
    }
}

use std::ops;

use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraph, SlotInfo, SlotType};
use bevy::render::render_resource::{Sampler, TextureView};
use bevy::render::renderer::RenderDevice;
use bevy::render::{RenderApp, RenderSet};
pub use strolle as st;

pub use self::camera::*;
pub use self::material::*;
use self::render_node::*;
use self::state::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<StrolleMaterial>();

        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>();
        let engine = st::Engine::new(render_device.wgpu_device());

        render_app.insert_resource(EngineResource(engine));
        render_app.insert_resource(SyncedState::default());

        // -------------------------- //
        // RenderSet::ExtractCommands //

        render_app.add_system(
            stages::extract::meshes
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::materials::<StandardMaterial>
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::materials::<StrolleMaterial>
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::instances::<StandardMaterial>
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::instances::<StrolleMaterial>
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::lights
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        render_app.add_system(
            stages::extract::cameras
                .in_schedule(ExtractSchedule)
                .in_set(RenderSet::ExtractCommands),
        );

        // ------------------ //
        // RenderSet::Prepare //

        render_app
            .add_system(stages::prepare::meshes.in_set(RenderSet::Prepare));

        render_app.add_system(
            stages::prepare::materials::<StandardMaterial>
                .in_set(RenderSet::Prepare),
        );

        render_app.add_system(
            stages::prepare::materials::<StrolleMaterial>
                .in_set(RenderSet::Prepare),
        );

        render_app.add_system(
            stages::prepare::instances::<StandardMaterial>
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials::<StandardMaterial>),
        );

        render_app.add_system(
            stages::prepare::instances::<StrolleMaterial>
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials::<StrolleMaterial>),
        );

        render_app
            .add_system(stages::prepare::lights.in_set(RenderSet::Prepare));

        // ---------------- //
        // RenderSet::Queue //

        render_app.add_system(stages::queue::cameras.in_set(RenderSet::Queue));

        render_app.add_system(
            stages::queue::write
                .in_set(RenderSet::Queue)
                .after(stages::queue::cameras),
        );

        // -----

        let render_node = RenderNode::new(&mut render_app.world);
        let upscaling_node = UpscalingNode::new(&mut render_app.world);
        let mut graph = RenderGraph::default();

        let input_node_id = graph.set_input(vec![SlotInfo::new(
            core_3d::graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);

        graph.add_node(graph::node::RENDER, render_node);
        graph.add_node(core_3d::graph::node::UPSCALING, upscaling_node);

        graph.add_slot_edge(
            input_node_id,
            core_3d::graph::input::VIEW_ENTITY,
            graph::node::RENDER,
            RenderNode::IN_VIEW,
        );

        graph.add_slot_edge(
            input_node_id,
            core_3d::graph::input::VIEW_ENTITY,
            core_3d::graph::node::UPSCALING,
            UpscalingNode::IN_VIEW,
        );

        graph.add_node_edge(
            graph::node::RENDER,
            core_3d::graph::node::UPSCALING,
        );

        render_app
            .world
            .resource_mut::<RenderGraph>()
            .add_sub_graph(graph::NAME, graph);
    }
}

#[derive(Resource)]
struct EngineResource(st::Engine<EngineParams>);

#[derive(Clone, Debug)]
struct EngineParams;

impl st::Params for EngineParams {
    type ImageHandle = Handle<Image>;
    type ImageSampler = Sampler;
    type ImageTexture = TextureView;
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
