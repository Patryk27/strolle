mod camera;
mod debug;
mod event;
pub mod graph;
mod rendering_node;
mod stages;
mod state;
mod sun;
mod utils;

pub mod prelude {
    pub use crate::*;
}

use std::ops;

use bevy::prelude::*;
use bevy::render::render_graph::RenderLabel;
use bevy::render::render_resource::Texture;
use bevy::render::renderer::RenderDevice;
use bevy::render::RenderApp;
pub use strolle as st;

pub use self::camera::*;
pub use self::debug::*;
pub use self::event::*;
pub(crate) use self::rendering_node::*;
pub(crate) use self::state::*;
pub use self::sun::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StrolleEvent>();
        app.insert_resource(StrolleSun::default());

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.insert_resource(SyncedState::default());

            stages::setup(render_app);
            graph::setup(render_app);
        }
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world().resource::<RenderDevice>();
        let engine = st::Engine::new(render_device.wgpu_device());

        render_app.insert_resource(EngineResource(engine));
    }
}

#[derive(Resource)]
struct EngineResource(st::Engine<EngineParams>);

#[derive(Clone, Debug)]
struct EngineParams;

impl st::Params for EngineParams {
    type ImageHandle = AssetId<Image>;
    type ImageTexture = Texture;
    type InstanceHandle = Entity;
    type LightHandle = Entity;
    type MaterialHandle = AssetId<StandardMaterial>;
    type MeshHandle = AssetId<Mesh>;
}

impl ops::Deref for EngineResource {
    type Target = st::Engine<EngineParams>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for EngineResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
