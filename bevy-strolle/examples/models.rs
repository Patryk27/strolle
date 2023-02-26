#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_obj::ObjPlugin;
use bevy_strolle::prelude::*;
use lazy_static::lazy_static;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

lazy_static! {
    static ref MODELS: Vec<(&'static str, Transform)> = vec![
        (
            "ferris.obj",
            Transform::from_scale(Vec3::splat(7.5))
                .with_translation(vec3(0.0, 2.0, 0.0))
                .with_rotation(Quat::from_rotation_y(-0.5))
        ),
        (
            "buddha.obj",
            Transform::from_scale(Vec3::splat(50.0))
                .with_translation(vec3(0.5, -2.5, 0.0))
        ),
        (
            "bunny.obj",
            Transform::from_scale(Vec3::splat(50.0))
                .with_translation(vec3(1.0, 0.5, 0.0))
        ),
        (
            "dragon.obj",
            Transform::from_scale(Vec3::splat(0.06))
                .with_translation(vec3(1.0, 5.0, 0.0))
                .with_rotation(Quat::from_rotation_y(PI)),
        ),
    ];
}

fn main() {
    common::unzip_assets();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 512.0,
                height: 512.0,
                mode: WindowMode::Windowed,
                ..default()
            },
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(ObjPlugin)
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(process_input)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 6.0),
        ..default()
    });

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            ..default()
        })
        .insert(OrbitCameraBundle::new(
            {
                let mut controller = OrbitCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.2;
                controller.mouse_translate_sensitivity = Vec2::ONE * 0.5;
                controller
            },
            vec3(-10.0, 10.0, 10.0),
            vec3(0.0, 5.0, 0.0),
        ));

    commands.insert_resource(State::default());
}

#[derive(Default, Resource)]
struct State {
    model_id: usize,
    model_entity: Option<Entity>,
}

fn process_input(
    mut commands: Commands,
    assets: Res<AssetServer>,
    keys: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
) {
    let mut reload_model = false;

    if keys.just_pressed(KeyCode::Left) {
        reload_model = true;

        state.model_id =
            state.model_id.checked_sub(1).unwrap_or(MODELS.len() - 1);
    }

    if keys.just_pressed(KeyCode::Right) {
        reload_model = true;
        state.model_id = (state.model_id + 1) % MODELS.len();
    }

    if state.model_entity.is_none() || reload_model {
        if let Some(entity) = state.model_entity.take() {
            commands.entity(entity).despawn();
        }

        let entity = commands
            .spawn(PbrBundle {
                mesh: assets.load(MODELS[state.model_id].0),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(1.0, 1.0, 1.0),
                    ..default()
                }),
                transform: MODELS[state.model_id].1,
                ..default()
            })
            .id();

        state.model_entity = Some(entity);
    }
}
