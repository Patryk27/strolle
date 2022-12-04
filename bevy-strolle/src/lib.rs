mod geometry_manager;
mod main_pass;

use bevy::core::Zeroable;
use bevy::core_pipeline::core_3d;
use bevy::prelude::*;
use bevy::render::render_graph::{RenderGraph, SlotInfo, SlotType};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::{RenderApp, RenderStage};
use strolle as st;

use self::geometry_manager::*;
use crate::main_pass::MainPass;

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
    materials: st::Materials,
}

#[derive(Resource)]
struct Strolle(pub st::Strolle);

pub struct StrollePlugin;

impl Plugin for StrollePlugin {
    fn build(&self, app: &mut App) {
        let mut render_app = app.get_sub_app_mut(RenderApp).unwrap();
        init_strolle(&mut render_app);
        render_app.insert_resource(State {
            geometry: Default::default(),
            camera: st::Camera::zeroed(), // TODO probably shouldn't be zeroed?
            lights: Default::default(),
            materials: Default::default(),
        });

        let mut sub_graph = RenderGraph::default();

        render_app.add_system_to_stage(RenderStage::Extract, extract::geometry);
        render_app.add_system_to_stage(RenderStage::Extract, extract::lights);
        render_app.add_system_to_stage(RenderStage::Extract, extract::camera);

        render_app.add_system_to_stage(RenderStage::Queue, queue);

        let input_node_id = sub_graph.set_input(vec![SlotInfo::new(
            core_3d::graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);

        sub_graph.add_node(
            graph::node::RENDER,
            MainPass::new(&mut render_app.world),
        );

        sub_graph
            .add_slot_edge(
                input_node_id,
                core_3d::graph::input::VIEW_ENTITY,
                graph::node::RENDER,
                MainPass::IN_VIEW,
            )
            .unwrap();

        render_app
            .world
            .resource_mut::<RenderGraph>()
            .add_sub_graph(graph::NAME, sub_graph);
    }
}

mod extract {
    use std::f32::consts::PI;

    use bevy::math::{vec2, vec3};
    use bevy::pbr::LightEntity;
    use bevy::prelude::*;
    use bevy::render::Extract;
    use strolle as st;

    use crate::State;

    pub(super) fn geometry(
        mut state: ResMut<State>,
        meshes: Extract<Res<Assets<Mesh>>>,
        models: Extract<
            Query<(Entity, &Handle<Mesh>, &Transform), Added<Handle<Mesh>>>,
        >,
    ) {
        let state = &mut *state;

        for (entity, mesh, &transform) in models.iter() {
            let mesh = meshes.get(mesh).unwrap();
            let transform = transform.compute_matrix();

            state.geometry.builder().add(entity, mesh, transform);
        }
    }

    pub(super) fn lights(
        mut state: ResMut<State>,
        lights: Extract<Query<(&LightEntity, &GlobalTransform)>>,
    ) {
        let state = &mut *state;

        state.lights = Default::default();

        for (_, transform) in lights.iter() {
            state.lights.push(st::Light::point(
                transform.translation(),
                vec3(1.0, 1.0, 1.0),
                1.0,
            ));
        }
    }

    pub(super) fn camera(
        mut state: ResMut<State>,
        cameras: Extract<Query<(&Camera, &GlobalTransform)>>,
    ) {
        for (camera, transform) in cameras.iter() {
            if !camera.is_active {
                continue;
            }

            state.camera = st::Camera::new(
                transform.translation(),
                transform.translation() + transform.forward(),
                transform.up(),
                1.0,
                vec2(256.0, 256.0),
                PI / 2.0,
            );
        }
    }
}

fn init_strolle(app: &mut App) {
    let render_device = app.world.get_resource::<RenderDevice>().unwrap();
    let render_queue = app.world.get_resource::<RenderQueue>().unwrap();

    let strolle = st::Strolle::new(
        render_device.wgpu_device(),
        render_queue.0.as_ref(),
        320,
        180,
        &[0; 2048 * 2048 * 4],
    );

    app.insert_resource(Strolle(strolle));
}

fn queue(
    strolle: Res<Strolle>,
    state: Res<State>,
    render_queue: Res<RenderQueue>,
) {
    strolle.0.update(
        render_queue.0.as_ref(),
        // &state.geometry.static_geo,
        // state.geometry.static_geo_index.as_ref().unwrap(),
        &state.geometry.dynamic_geo,
        &state.geometry.uvs,
        &state.camera,
        &state.lights,
        &state.materials,
    );
}
