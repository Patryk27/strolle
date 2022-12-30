use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_obj::ObjPlugin;
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
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            reflectance: 0.0,
            ..default()
        }),
        transform: Transform::from_translation(vec3(0.0, -25.0, 0.0)),
        ..default()
    });

    for i in 0..10 {
        for j in 0..10 {
            commands
                .spawn(PbrBundle {
                    mesh: assets.load("bunny.obj"),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(1.0, 1.0, 1.0),
                        reflectance: 0.0,
                        ..default()
                    }),
                    transform: Transform::from_scale(Vec3::splat(20.0)),
                    ..default()
                })
                .insert(Animated { i, j });
        }
    }

    for (idx, color) in [Color::RED, Color::GREEN, Color::BLUE]
        .into_iter()
        .enumerate()
    {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color,
                intensity: 3000.0,
                range: 1000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(
                Quat::from_rotation_y(2.0 * (idx as f32) / 3.0)
                    * vec3(2.0, 0.0, 0.0),
            ),
            ..default()
        });
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
            Vec3::new(-40.0, 50.0, -90.0),
            Vec3::new(0.0, -5.0, 0.0),
        ));
}

fn animate(
    time: Res<Time>,
    mut objects: Query<(Entity, &mut Transform, &Animated)>,
) {
    let tt = time.elapsed_seconds() / 3.0;

    for (entity, mut transform, animated) in objects.iter_mut() {
        let seed = (entity.reflect_hash().unwrap() % 100) as f32 / 100.0;
        let phi = PI * (animated.i as f32 + 1.0) / 11.0;
        let theta = 2.0 * PI * (animated.j as f32) / 10.0;

        transform.translation =
            vec3(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin())
                * 20.0;

        transform.translation = Quat::from_rotation_x(tt)
            * Quat::from_rotation_z(tt * 2.0)
            * transform.translation;

        transform.rotation = Quat::from_rotation_x(tt * (1.0 + seed))
            * Quat::from_rotation_y(tt * 2.0 * (1.0 + seed))
            * Quat::from_rotation_z(tt + 2.0 * (1.0 + seed));

        transform.translation.y += 5.0;
    }
}

#[derive(Component)]
struct Animated {
    i: u32,
    j: u32,
}
