use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_strolle::StrollePlugin;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(animate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.2),
            perceptual_roughness: 0.0,
            ..default()
        }),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.7, 0.6),
                perceptual_roughness: 0.2,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Animated { phase: 0.0 });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.8, 0.7, 0.6, 0.25),
                perceptual_roughness: 0.2,
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Animated { phase: PI });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 8.0, -4.0),
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
            Vec3::new(-10.0, 5.0, 10.0),
            Vec3::ZERO,
        ));
}

fn animate(time: Res<Time>, mut objects: Query<(&mut Transform, &Animated)>) {
    const RADIUS: f32 = 3.0;

    let tt = time.elapsed_seconds();

    for (mut transform, animated) in objects.iter_mut() {
        transform.translation.x = RADIUS * (animated.phase + tt).sin();
        transform.translation.z = RADIUS * (animated.phase + tt).cos();
        transform.translation.y = 0.5 + tt.sin().abs() * 1.8;

        transform.rotation =
            Quat::from_rotation_x(tt) * Quat::from_rotation_y(tt / 1.5);
    }
}

#[derive(Component)]
struct Animated {
    phase: f32,
}
