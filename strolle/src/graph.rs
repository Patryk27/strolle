pub const REFERENCE: &str = "strolle_reference";
pub const BVH_HEATMAP: &str = "strolle_bvh_heatmap";

pub mod node {
    pub const BVH_HEATMAP: &str = "strolle_bvh_heatmap";

    pub const INDIRECT_TRACING: &str = "strolle_indirect_tracing";

    pub const INDIRECT_SHADING: &str = "strolle_indirect_shading";

    pub const INDIRECT_DIFFUSE_TEMPORAL_RESAMPLING: &str =
        "strolle_indirect_diffuse_temporal_resampling";

    pub const INDIRECT_DIFFUSE_SPATIAL_RESAMPLING: &str =
        "strolle_indirect_diffuse_spatial_resampling";

    pub const INDIRECT_DIFFUSE_RESOLVING: &str =
        "strolle_indirect_diffuse_resolving";

    pub const INDIRECT_DIFFUSE_DENOISING: &str =
        "strolle_indirect_diffuse_denoising";

    pub const FRAME_COMPOSITION: &str = "strolle_frame_composition";

    pub const UPSCALING: &str = "strolle_upscaling";
}

use bevy::app::App;
use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::core_3d::CORE_3D;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};

use crate::pipelines::*;

pub(crate) fn setup(render_app: &mut App) {
    render_app
        .add_render_graph_node::<ViewNodeRunner<IndirectTracingNode>>(
            CORE_3D,
            node::INDIRECT_TRACING,
        )
        .add_render_graph_node::<ViewNodeRunner<IndirectShadingNode>>(
            CORE_3D,
            node::INDIRECT_SHADING,
        )
        .add_render_graph_node::<ViewNodeRunner<FrameCompositionNode>>(
            CORE_3D,
            node::FRAME_COMPOSITION,
        )
        .add_render_graph_edges(
            CORE_3D,
            &[
                core_3d::graph::node::END_MAIN_PASS,
                node::INDIRECT_TRACING,
                node::INDIRECT_SHADING,
                node::FRAME_COMPOSITION,
                core_3d::graph::node::BLOOM,
                core_3d::graph::node::TONEMAPPING,
            ],
        );

    // ---

    render_app
        .add_render_sub_graph(BVH_HEATMAP)
        .add_render_graph_node::<ViewNodeRunner<BvhHeatmapNode>>(
            BVH_HEATMAP,
            node::BVH_HEATMAP,
        )
        .add_render_graph_node::<ViewNodeRunner<FrameCompositionNode>>(
            BVH_HEATMAP,
            node::FRAME_COMPOSITION,
        )
        .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
            BVH_HEATMAP,
            node::UPSCALING,
        )
        .add_render_graph_edges(
            BVH_HEATMAP,
            &[node::BVH_HEATMAP, node::FRAME_COMPOSITION, node::UPSCALING],
        );
}
