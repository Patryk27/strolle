#[path = "_common.rs"]
mod common;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::WindowResolution;
use bevy_obj::ObjPlugin;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    common::unzip_assets();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(512.0, 512.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(ObjPlugin)
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(animate)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(SceneBundle {
        scene: assets.load("cornell/scene.gltf#Scene0"),
        ..Default::default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 50.0,
            shadows_enabled: true,
            ..default()
        },
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
            vec3(0.0, 1.0, 3.2),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ));
}

fn animate(
    time: Res<Time>,
    mut light: Query<&mut Transform, With<PointLight>>,
) {
    light.single_mut().translation =
        vec3(0.0f32.sin() / 2.0, 1.5, 0.0f32.cos() / 2.0);
}
