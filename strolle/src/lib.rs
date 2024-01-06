#![feature(hash_raw_entry)]
#![feature(lint_reasons)]

mod bvh;
mod event;
pub mod graph;
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

use bevy::app::{App, Plugin};
use bevy::render::render_resource::Texture;
use bevy::render::RenderApp;
pub(crate) use strolle_gpu as gpu;

pub(crate) use self::bvh::*;
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
pub(crate) use crate::noise::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Event>();
        app.insert_resource(Sun::default());

        let render_app = app.sub_app_mut(RenderApp);

        stages::setup(render_app);
        graph::setup(render_app);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<Bvh>()
            .init_resource::<State>()
            .init_resource::<Images>()
            .init_resource::<Instances>()
            .init_resource::<Lights>()
            .init_resource::<Materials>()
            .init_resource::<Meshes>()
            .init_resource::<Noise>()
            .init_resource::<Sun>()
            .init_resource::<Triangles>();

        render_app
            .init_resource::<BvhHeatmapPipeline>()
            .init_resource::<FrameCompositionPipeline>()
            .init_resource::<IndirectShadingPipeline>()
            .init_resource::<IndirectTracingPipeline>();
    }
}
