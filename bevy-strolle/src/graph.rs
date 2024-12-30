use bevy::core_pipeline::fxaa::FxaaNode;
use bevy::core_pipeline::tonemapping::TonemappingNode;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::prelude::*;
use bevy::render::render_graph::{
    RenderGraphApp, RenderSubGraph, ViewNodeRunner,
};

use crate::prelude::RenderLabel;
use crate::RenderingNode;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct StrolleGraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum StrolleNode {
    RENDERING,
    TONEMAPPING,
    FXAA,
    UPSCALING,
}

pub(crate) fn setup(render_app: &mut SubApp) {
    render_app
        .add_render_sub_graph(StrolleGraph)
        .add_render_graph_node::<ViewNodeRunner<RenderingNode>>(
            StrolleGraph,
            StrolleNode::RENDERING,
        )
        .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(
            StrolleGraph,
            StrolleNode::TONEMAPPING,
        )
        .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
            StrolleGraph,
            StrolleNode::UPSCALING,
        )
        .add_render_graph_node::<ViewNodeRunner<FxaaNode>>(
            StrolleGraph,
            StrolleNode::FXAA,
        )
        .add_render_graph_edges(
            StrolleGraph,
            (
                StrolleNode::RENDERING,
                StrolleNode::FXAA,
                StrolleNode::TONEMAPPING,
                StrolleNode::UPSCALING,
            ),
        );
}
