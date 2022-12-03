mod geometry_manager;

use std::f32::consts::PI;

use bevy::math::{vec2, vec3};
use bevy::pbr::LightEntity;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraph, SlotInfo, SlotType};
use bevy::render::RenderApp;
use strolle as st;

use self::geometry_manager::*;

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        use bevy::core_pipeline::core_3d;

        app.add_system(sync_geometry);
        app.add_system(sync_lights);
        app.add_system(sync_camera);

        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        let mut sub_graph = RenderGraph::default();

        sub_graph.set_input(vec![SlotInfo::new(
            core_3d::graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);

        // TODO
        // sub_graph.add_node(
        //     graph::node::RENDER,
        //     RenderNode::new(&mut render_app.world),
        // );

        render_app
            .world
            .resource_mut::<RenderGraph>()
            .add_sub_graph(graph::NAME, sub_graph);
    }
}

pub struct Static;

pub mod graph {
    pub const NAME: &str = "strolle";

    pub mod node {
        pub const RENDER: &str = "strolle_render";
    }
}

#[derive(Resource)]
struct State {
    geometry: GeometryManager,
    camera: st::Camera,
    lights: st::Lights,
}

fn sync_geometry(
    mut state: ResMut<State>,
    meshes: Res<Assets<Mesh>>,
    models: Query<(Entity, &Handle<Mesh>, &Transform), Added<Handle<Mesh>>>,
) {
    let state = &mut *state;

    for (entity, mesh, &transform) in models.iter() {
        let mesh = meshes.get(mesh).unwrap();
        let transform = transform.compute_matrix();

        state.geometry.builder().add(entity, mesh, transform);
    }
}

fn sync_lights(
    mut state: ResMut<State>,
    lights: Query<(&LightEntity, &Transform)>,
) {
    let state = &mut *state;

    state.lights = Default::default();

    for (_, transform) in lights.iter() {
        state.lights.push(st::Light::point(
            transform.translation,
            vec3(1.0, 1.0, 1.0),
            1.0,
        ));
    }
}

fn sync_camera(mut state: ResMut<State>) {
    state.camera = st::Camera::new(
        vec3(-2.0, 2.5, 5.0),
        Vec3::ZERO,
        Vec3::Y,
        1.0,
        vec2(256.0, 256.0),
        PI / 2.0,
    );
}
