#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_obj::ObjPlugin;
use bevy_strolle::StrollePlugin;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

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
        .add_system(animate)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            reflectance: 0.0,
            ..default()
        }),
        transform: Transform::from_translation(bevy::math::vec3(
            0.0, -2.5, 0.0,
        )),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: assets.load("nefertiti.obj"),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        }),
        transform: Transform::from_scale(Vec3::splat(0.01)).with_rotation(
            Quat::from_rotation_y(-PI / 2.0) * Quat::from_rotation_x(-PI / 2.0),
        ),
        ..default()
    });

    for (color, phase) in [
        (Color::RED, 0.0),
        (Color::GREEN, 2.0 * PI / 3.0),
        (Color::BLUE, 4.0 * PI / 3.0),
    ] {
        commands
            .spawn(PointLightBundle {
                point_light: PointLight {
                    color,
                    intensity: 1500.0,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            })
            .insert(Animated { phase });
    }

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
            Vec3::new(-8.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ));
}

fn animate(time: Res<Time>, mut objects: Query<(&mut Transform, &Animated)>) {
    const RADIUS: f32 = 10.0;

    let tt = time.elapsed_seconds();

    for (mut transform, animated) in objects.iter_mut() {
        transform.translation.x = RADIUS * (animated.phase + tt).sin();
        transform.translation.z = RADIUS * (animated.phase + tt).cos();
        transform.translation.y = 5.0 + 2.0 * (animated.phase + tt).sin();
    }
}

#[derive(Component)]
struct Animated {
    phase: f32,
}
