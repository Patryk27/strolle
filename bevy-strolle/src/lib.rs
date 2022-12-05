mod extract;
mod render_node;
mod state;

pub mod graph {
    pub const NAME: &str = "strolle";

    pub mod node {
        pub const RENDER: &str = "strolle_render";
    }
}

use std::ops;

use bevy::core_pipeline::core_3d;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraph, SlotInfo, SlotType};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{RenderApp, RenderStage};
use strolle as st;

use self::render_node::*;
use self::state::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world.resource::<RenderDevice>();
        let render_queue = render_app.world.resource::<RenderQueue>();

        let strolle = st::Strolle::new(
            render_device.wgpu_device(),
            render_queue.0.as_ref(),
            &[0; 2048 * 2048 * 4],
        );

        render_app.insert_resource(StrolleRes(strolle));
        render_app.insert_resource(ExtractedState::default());

        // -----

        render_app.add_system_to_stage(RenderStage::Extract, extract::geometry);
        render_app.add_system_to_stage(RenderStage::Extract, extract::lights);
        render_app.add_system_to_stage(RenderStage::Extract, extract::camera);
        render_app.add_system_to_stage(RenderStage::Queue, queue);

        // -----

        let render_node = RenderNode::new(&mut render_app.world);
        let mut sub_graph = RenderGraph::default();

        let input_node_id = sub_graph.set_input(vec![SlotInfo::new(
            core_3d::graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);

        sub_graph.add_node(graph::node::RENDER, render_node);

        sub_graph
            .add_slot_edge(
                input_node_id,
                core_3d::graph::input::VIEW_ENTITY,
                graph::node::RENDER,
                RenderNode::IN_VIEW,
            )
            .unwrap();

        render_app
            .world
            .resource_mut::<RenderGraph>()
            .add_sub_graph(graph::NAME, sub_graph);
    }
}

#[derive(Resource)]
struct StrolleRes(st::Strolle);

impl ops::Deref for StrolleRes {
    type Target = st::Strolle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for StrolleRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn queue(
    strolle: Res<StrolleRes>,
    mut state: ResMut<ExtractedState>,
    queue: Res<RenderQueue>,
) {
    state.enqueue(&strolle, &*queue);
}
