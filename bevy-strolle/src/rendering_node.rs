use bevy::prelude::*;
use bevy::render::render_graph::{NodeRunError, RenderGraphContext, ViewNode};
use bevy::render::renderer::RenderContext;
use bevy::render::view::ViewTarget;

use crate::{EngineResource, SyncedState};

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
        let engine = world.resource::<EngineResource>();
        let state = world.resource::<SyncedState>();

        let Some(camera) = state.cameras.get(&entity) else {
            println!("No camera");
            return Ok(());
        };

        //println!("Starting render for camera {:?}", entity);

        engine.render_camera(
            camera.handle,
            render_context.command_encoder(),
            target.main_texture_view(),
        );

        Ok(())
    }
}
