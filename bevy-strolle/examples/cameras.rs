#[path = "_common.rs"]
mod common;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, Viewport};
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};
use bevy_strolle::prelude::*;
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
        .add_plugin(LookTransformPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(update_cameras)
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
        transform: Transform::from_translation(vec3(0.0, 1.5, 0.0)),
        ..default()
    });

    // -----

    let transform = Transform::from_xyz(0.0, 1.0, 3.2)
        .looking_at(vec3(0.0, 1.0, 0.0), Vec3::Y);

    let bevy_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            transform,
            ..default()
        })
        .id();

    let strolle_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .id();

    let strolle_direct_lightning_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .insert(StrolleCamera {
            config: st::Camera {
                mode: st::CameraMode::DisplayDirectLightning,
                ..default()
            },
        })
        .id();

    let strolle_indirect_lightning_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                order: 3,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .insert(StrolleCamera {
            config: st::Camera {
                mode: st::CameraMode::DisplayIndirectLightning,
                ..default()
            },
        })
        .id();

    // -----

    commands.insert_resource(State {
        cameras: [
            bevy_camera,
            strolle_camera,
            strolle_direct_lightning_camera,
            strolle_indirect_lightning_camera,
        ],
    });
}

#[derive(Resource)]
struct State {
    cameras: [Entity; 4],
}

fn update_cameras(
    state: Res<State>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera>,
) {
    let window = windows.single();

    let (window_width, window_height) =
        (window.physical_width(), window.physical_height());

    let mut physical_position = uvec2(0, 0);
    let physical_size = uvec2(window_width / 2, window_height / 2);

    for &camera in &state.cameras {
        cameras.get_mut(camera).unwrap().viewport = Some(Viewport {
            physical_position,
            physical_size,
            ..default()
        });

        physical_position.x += physical_size.x;

        if physical_position.x + physical_size.x > window_width {
            physical_position.x = 0;
            physical_position.y += physical_size.y;
        }
    }
}
