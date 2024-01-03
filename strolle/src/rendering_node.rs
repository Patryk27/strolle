use bevy::prelude::*;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy::render::renderer::RenderContext;
use bevy::render::view::ViewTarget;

#[derive(Default)]
pub struct RenderingNode;

impl ViewNode for RenderingNode {
    type ViewQuery = &'static ViewTarget;

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        target: &ViewTarget,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let entity = graph.view_entity();

        todo!();
    }
}
