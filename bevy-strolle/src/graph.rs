pub const NAME: &str = "strolle";

pub mod node {
    pub const RENDERING: &str = "strolle_rendering";
    pub const TONEMAPPING: &str = "strolle_tonemapping";
    pub const FXAA: &str = "strolle_fxaa";
    pub const UPSCALING: &str = "strolle_upscaling";
}

use bevy::core_pipeline::fxaa::FxaaNode;
use bevy::core_pipeline::tonemapping::TonemappingNode;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};

use crate::RenderingNode;

pub(crate) fn setup(render_app: &mut App) {
    render_app
        .add_render_sub_graph(NAME)
        .add_render_graph_node::<ViewNodeRunner<RenderingNode>>(
            NAME,
            node::RENDERING,
        )
        .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(
            NAME,
            node::TONEMAPPING,
        )
        .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
            NAME,
            node::UPSCALING,
        )
        .add_render_graph_node::<ViewNodeRunner<FxaaNode>>(
            NAME,
            node::FXAA,
        )
        .add_render_graph_edges(
            NAME,
            &[node::RENDERING,node::FXAA, node::TONEMAPPING, node::UPSCALING],
        );
}
