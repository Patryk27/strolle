use bevy::prelude::*;
use bevy::render::render_graph::{
    Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType,
};
use bevy::render::renderer::RenderContext;
use bevy::render::view::{ExtractedView, ViewTarget};

use crate::SyncedState;

pub struct RenderNode {
    query: QueryState<&'static ViewTarget, With<ExtractedView>>,
}

impl RenderNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for RenderNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world)
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let entity = graph.get_input_entity(Self::IN_VIEW)?;

        let Ok(target) = self.query.get_manual(world, entity) else {
            return Ok(());
        };

        let state = world.resource::<SyncedState>();

        let Some(view) = state.views.get(&entity) else {
            return Ok(());
        };

        view.viewport
            .render(&mut render_context.command_encoder, target.main_texture());

        Ok(())
    }
}
