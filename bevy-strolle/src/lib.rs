mod render_node;
mod stages;
mod state;
mod utils;

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
use bevy::render::renderer::RenderDevice;
use bevy::render::{RenderApp, RenderStage};
use strolle as st;

use self::render_node::*;
use self::state::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>();
        let engine = st::Engine::new(render_device.wgpu_device());

        render_app.insert_resource(EngineRes(engine));
        render_app.insert_resource(SyncedState::default());

        // -------------------- //
        // RenderStage::Extract //

        render_app
            .add_system_to_stage(RenderStage::Extract, stages::extract::meshes);

        render_app.add_system_to_stage(
            RenderStage::Extract,
            stages::extract::materials,
        );

        render_app.add_system_to_stage(
            RenderStage::Extract,
            stages::extract::instances,
        );

        render_app
            .add_system_to_stage(RenderStage::Extract, stages::extract::lights);

        render_app.add_system_to_stage(
            RenderStage::Extract,
            stages::extract::cameras,
        );

        // -------------------- //
        // RenderStage::Prepare //

        render_app
            .add_system_to_stage(RenderStage::Prepare, stages::prepare::meshes);

        render_app.add_system_to_stage(
            RenderStage::Prepare,
            stages::prepare::materials,
        );

        render_app.add_system_to_stage(
            RenderStage::Prepare,
            stages::prepare::instances
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials),
        );

        render_app
            .add_system_to_stage(RenderStage::Prepare, stages::prepare::lights);

        // ------------------ //
        // RenderStage::Queue //

        render_app
            .add_system_to_stage(RenderStage::Queue, stages::queue::viewports);

        render_app.add_system_to_stage(
            RenderStage::Queue,
            stages::queue::write.after(stages::queue::viewports),
        );

        // -----

        let render_node = RenderNode::new(&mut render_app.world);
        let upscaling_node = UpscalingNode::new(&mut render_app.world);
        let mut sub_graph = RenderGraph::default();

        let input_node_id = sub_graph.set_input(vec![SlotInfo::new(
            core_3d::graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);

        sub_graph.add_node(graph::node::RENDER, render_node);
        sub_graph.add_node(core_3d::graph::node::UPSCALING, upscaling_node);

        sub_graph
            .add_slot_edge(
                input_node_id,
                core_3d::graph::input::VIEW_ENTITY,
                graph::node::RENDER,
                RenderNode::IN_VIEW,
            )
            .unwrap();

        sub_graph
            .add_slot_edge(
                input_node_id,
                core_3d::graph::input::VIEW_ENTITY,
                core_3d::graph::node::UPSCALING,
                UpscalingNode::IN_VIEW,
            )
            .unwrap();

        sub_graph
            .add_node_edge(graph::node::RENDER, core_3d::graph::node::UPSCALING)
            .unwrap();

        render_app
            .world
            .resource_mut::<RenderGraph>()
            .add_sub_graph(graph::NAME, sub_graph);
    }
}

#[derive(Resource)]
struct EngineRes(st::Engine<Self>);

impl st::EngineParams for EngineRes {
    type MeshHandle = Handle<Mesh>;
    type LightHandle = Entity;
    type MaterialHandle = Handle<StandardMaterial>;
}

impl ops::Deref for EngineRes {
    type Target = st::Engine<Self>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for EngineRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
