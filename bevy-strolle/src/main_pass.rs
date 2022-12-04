use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::render_graph::{Node, SlotInfo, SlotType};
use bevy::render::render_resource::{LoadOp, Operations, RenderPassDescriptor};
use bevy::render::view::{ExtractedView, ViewTarget};

pub struct MainPass {
    query: QueryState<
        (
            &'static ExtractedCamera,
            &'static Camera3d,
            &'static ViewTarget,
        ),
        With<ExtractedView>,
    >,
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

        let Ok((_, camera_3d, target)) = self.query.get_manual(world, entity) else {
            return Ok(())
        };

        let strolle = world.resource::<super::Strolle>();

        let color_op = Operations {
            load: match camera_3d.clear_color {
                ClearColorConfig::Default => {
                    LoadOp::Clear(world.resource::<ClearColor>().0.into())
                }
                ClearColorConfig::Custom(color) => LoadOp::Clear(color.into()),
                ClearColorConfig::None => LoadOp::Load,
            },
            store: true,
        };

        let color_attachment = target.get_unsampled_color_attachment(color_op);

        strolle
            .0
            .render(&mut render_context.command_encoder, color_attachment);

        Ok(())
    }
}
