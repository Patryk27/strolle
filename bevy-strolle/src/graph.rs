use bevy::core_pipeline::fxaa::FxaaNode;
use bevy::core_pipeline::tonemapping::TonemappingNode;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::prelude::*;
use bevy::render::render_graph::{
    RenderGraphApp, RenderLabel, RenderSubGraph, ViewNodeRunner,
};

use crate::RenderingNode;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct StrolleGraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub enum StrolleNode {
    Rendering,
    ToneMapping,
    Fxaa,
    Upscaling,
}

pub(crate) fn setup(app: &mut SubApp) {
    app.add_render_sub_graph(StrolleGraph)
        .add_render_graph_node::<ViewNodeRunner<RenderingNode>>(
            StrolleGraph,
            StrolleNode::Rendering,
        )
        .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(
            StrolleGraph,
            StrolleNode::ToneMapping,
        )
        .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
            StrolleGraph,
            StrolleNode::Upscaling,
        )
        .add_render_graph_node::<ViewNodeRunner<FxaaNode>>(
            StrolleGraph,
            StrolleNode::Fxaa,
        )
        .add_render_graph_edges(
            StrolleGraph,
            (
                StrolleNode::Rendering,
                StrolleNode::Fxaa,
                StrolleNode::ToneMapping,
                StrolleNode::Upscaling,
            ),
        );
}
