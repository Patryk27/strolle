#![feature(hash_raw_entry)]
#![feature(lint_reasons)]

mod bvh;
mod camera;
mod event;
mod image;
mod images;
mod instance;
mod instances;
mod light;
mod lights;
mod material;
mod materials;
mod mesh;
mod mesh_triangle;
mod meshes;
mod noise;
mod pipelines;
mod stages;
mod state;
mod sun;
mod triangle;
mod triangles;
mod utils;

pub mod graph {
    pub const BVH_HEATMAP: &str = "strolle_bvh_heatmap";

    pub mod node {
        pub const RENDERING: &str = "strolle_rendering";
        pub const COMPOSING: &str = "strolle_composing";
        pub const UPSCALING: &str = "strolle_upscaling";
    }
}

use bevy::app::{App, Plugin};
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::Texture;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSet};
pub(crate) use strolle_gpu as gpu;

pub(crate) use self::bvh::*;
pub(crate) use self::camera::*;
pub(crate) use self::event::*;
pub use self::image::*;
pub(crate) use self::images::*;
pub use self::instance::*;
pub(crate) use self::instances::*;
pub use self::light::*;
pub(crate) use self::lights::*;
pub use self::material::*;
pub(crate) use self::materials::*;
pub use self::mesh::*;
pub use self::mesh_triangle::*;
pub(crate) use self::meshes::*;
pub(crate) use self::pipelines::*;
pub(crate) use self::state::*;
pub use self::sun::*;
pub(crate) use self::triangle::*;
pub(crate) use self::triangles::*;
pub(crate) use self::utils::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.insert_resource(Sun::default());

        let render_app = app.sub_app_mut(RenderApp);

        // -------------------------- //
        // RenderSet::ExtractCommands //

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::meshes.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::materials.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::instances.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::images.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::lights.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::cameras.in_set(RenderSet::ExtractCommands),
        );

        render_app.add_systems(
            ExtractSchedule,
            stages::extract::sun.in_set(RenderSet::ExtractCommands),
        );

        // ------------------ //
        // RenderSet::Prepare //

        render_app.add_systems(
            Render,
            stages::prepare::meshes.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::materials.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::instances
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::meshes)
                .after(stages::prepare::materials),
        );

        render_app.add_systems(
            Render,
            stages::prepare::images.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::lights.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::sun.in_set(RenderSet::Prepare),
        );

        render_app.add_systems(
            Render,
            stages::prepare::refresh
                .in_set(RenderSet::Prepare)
                .after(stages::prepare::instances),
        );

        render_app.add_systems(
            Render,
            (
                stages::prepare::buffers.in_set(RenderSet::PrepareResources),
                stages::prepare::textures.in_set(RenderSet::PrepareResources),
            )
                .chain(),
        );

        render_app.add_systems(
            Render,
            stages::prepare::flush.in_set(RenderSet::PrepareFlush),
        );

        // -----

        render_app
            .add_render_sub_graph(graph::BVH_HEATMAP)
            .add_render_graph_node::<ViewNodeRunner<BvhHeatmapNode>>(
                graph::BVH_HEATMAP,
                graph::node::RENDERING,
            )
            .add_render_graph_node::<ViewNodeRunner<FrameCompositionNode>>(
                graph::BVH_HEATMAP,
                graph::node::COMPOSING,
            )
            .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
                graph::BVH_HEATMAP,
                graph::node::UPSCALING,
            )
            .add_render_graph_edges(
                graph::BVH_HEATMAP,
                &[
                    graph::node::RENDERING,
                    graph::node::COMPOSING,
                    graph::node::UPSCALING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<Bvh>()
            .init_resource::<CamerasBuffers>()
            .init_resource::<Images>()
            .init_resource::<Instances>()
            .init_resource::<Lights>()
            .init_resource::<Materials>()
            .init_resource::<Meshes>()
            .init_resource::<Triangles>()
            .init_resource::<Sun>()
            .init_resource::<pipelines::BvhHeatmapPipeline>()
            .init_resource::<pipelines::FrameCompositionPipeline>();
    }
}
