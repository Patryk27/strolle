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
mod rendering_node;
mod shaders;
mod stages;
mod state;
mod sun;
mod triangle;
mod triangles;
mod utils;

pub mod prelude {
    // TODO
}

pub mod graph {
    pub const NAME: &str = "strolle";

    pub mod node {
        pub const RENDERING: &str = "strolle_rendering";
        pub const TONEMAPPING: &str = "strolle_tonemapping";
        pub const UPSCALING: &str = "strolle_upscaling";
    }
}

use bevy::app::{App, Plugin};
use bevy::core_pipeline::tonemapping::TonemappingNode;
use bevy::core_pipeline::upscaling::UpscalingNode;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::render::render_graph::{RenderGraphApp, ViewNodeRunner};
use bevy::render::render_resource::Texture;
use bevy::render::renderer::RenderDevice;
use bevy::render::{ExtractSchedule, Render, RenderApp, RenderSet};
pub(crate) use strolle_gpu as gpu;

pub(crate) use self::bvh::*;
pub use self::camera::*;
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
use self::rendering_node::*;
pub(crate) use self::shaders::*;
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

        // ---------------- //
        // RenderSet::Queue //

        // render_app.add_systems(
        //     Render,
        //     stages::queue::cameras.in_set(RenderSet::Queue),
        // );

        // render_app.add_systems(
        //     Render,
        //     stages::queue::write
        //         .in_set(RenderSet::Queue)
        //         .after(stages::queue::cameras),
        // );

        // -----

        render_app
            .add_render_sub_graph(graph::NAME)
            .add_render_graph_node::<ViewNodeRunner<RenderingNode>>(
                graph::NAME,
                graph::node::RENDERING,
            )
            .add_render_graph_node::<ViewNodeRunner<TonemappingNode>>(
                graph::NAME,
                graph::node::TONEMAPPING,
            )
            .add_render_graph_node::<ViewNodeRunner<UpscalingNode>>(
                graph::NAME,
                graph::node::UPSCALING,
            )
            .add_render_graph_edges(
                graph::NAME,
                &[
                    graph::node::RENDERING,
                    graph::node::TONEMAPPING,
                    graph::node::UPSCALING,
                ],
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        let device = render_app.world.resource::<RenderDevice>();
    }
}
