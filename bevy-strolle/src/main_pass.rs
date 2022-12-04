use bevy::prelude::*;
use bevy::render::render_graph::{Node, SlotInfo, SlotType};
use bevy::render::render_resource::{
    LoadOp, Operations, RenderPassColorAttachment,
};
use bevy::render::view::{ExtractedView, ViewTarget};

pub struct MainPass {
    query: QueryState<&'static ViewTarget, With<ExtractedView>>,
}

impl MainPass {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for MainPass {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(Self::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world)
    }

    fn run(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &bevy::prelude::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let entity = graph.get_input_entity(Self::IN_VIEW)?;

        let Ok(target) = self.query.get_manual(world, entity) else {
            return Ok(())
        };

        let strolle = world.resource::<super::Strolle>();

        let color_attachment = RenderPassColorAttachment {
            view: target.out_texture(),
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Default::default()),
                store: true,
            },
        };

        strolle
            .0
            .render(&mut render_context.command_encoder, color_attachment);

        Ok(())
    }
}
