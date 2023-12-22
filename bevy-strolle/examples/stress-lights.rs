use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(512.0, 512.0),
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            StrollePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update_lights)
        .run();
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun: ResMut<StrolleSun>,
) {
    let mut window = window.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // ---

    sun.altitude = -1.0;

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        })
        .insert(FpsCameraBundle::new(
            {
                let mut controller = FpsCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.35;
                controller.translate_sensitivity = 8.0;
                controller
            },
            vec3(0.0, 50.0, 80.0),
            vec3(0.0, 50.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ));

    // ---

    let mesh = meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0)));
    let material = materials.add(StandardMaterial::from(Color::WHITE));

    commands.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: material.clone(),
        transform: Transform::from_scale(vec3(1000.0, 1.0, 1000.0))
            .with_translation(vec3(0.0, -2.0, 0.0)),
        ..default()
    });

    let walls = [
        Transform::from_scale(vec3(1.0, 10.0, 10.0))
            .with_translation(vec3(-5.0, 5.0, 0.0)),
        Transform::from_scale(vec3(1.0, 10.0, 10.0))
            .with_translation(vec3(5.0, 5.0, 0.0)),
        Transform::from_scale(vec3(10.0, 10.0, 1.0))
            .with_translation(vec3(0.0, 5.0, -5.0)),
        Transform::from_scale(vec3(10.0, 1.0, 10.0))
            .with_translation(vec3(0.0, 10.0, 0.0)),
        Transform::from_scale(vec3(10.0, 1.0, 10.0))
            .with_translation(vec3(0.0, 0.0, 0.0)),
    ];

    let mut phase = 0.0;

    for x in -5..5 {
        for y in 0..10 {
            let x = x as f32 * 15.0;
            let y = y as f32 * 15.0;

            for mut wall in walls {
                wall.translation += vec3(x, y, 0.0);

                commands.spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: wall,
                    ..default()
                });
            }

            commands
                .spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::WHITE,
                        range: 20.0,
                        radius: 0.25,
                        intensity: 3000.0,
                        shadows_enabled: true,
                        ..default()
                    },
                    ..default()
                })
                .insert(Light {
                    anchor: vec3(x, 5.0 + y, 0.0),
                    phase,
                });

            phase += 123.456;
        }
    }
}

#[derive(Component, Debug)]
struct Light {
    anchor: Vec3,
    phase: f32,
}

fn update_lights(
    time: Res<Time>,
    mut lights: Query<(&mut Transform, &mut PointLight, &Light)>,
) {
    for (mut light_xform, mut light_data, light) in lights.iter_mut() {
        let t = light.phase * 2.0 * PI + time.elapsed_seconds();

        light_xform.translation =
            light.anchor + vec3(t.sin() * 4.0, t.cos() * 4.0, 0.0);

        light_data.color = Color::hsl((t * 90.0) % 360.0, 1.0, 0.5);
    }
}
