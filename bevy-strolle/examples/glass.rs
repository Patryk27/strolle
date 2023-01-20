use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_strolle::{StrolleMaterial, StrollePlugin};
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
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
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut strolle_materials: ResMut<Assets<StrolleMaterial>>,
) {
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 1.0,
        subdivisions: 6,
    }));

    commands.spawn(MaterialMeshBundle {
        mesh: sphere_mesh.clone(),
        material: strolle_materials.add(StrolleMaterial {
            parent: StandardMaterial {
                base_color: Color::rgba(0.2, 0.0, 0.0, 0.8),
                alpha_mode: AlphaMode::Blend,
                ..default()
            },
            refraction: 1.1,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 2.0, 0.0)
            .with_scale(Vec3::splat(2.0)),
        ..default()
    });

    for (translation, base_color) in [
        (vec3(-3.5, 1.5, -4.5), Color::RED),
        (vec3(0.0, 1.5, -4.5), Color::GREEN),
        (vec3(3.5, 1.5, -4.5), Color::BLUE),
    ] {
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: materials.add(StandardMaterial {
                base_color,
                ..default()
            }),
            transform: Transform::from_translation(translation),
            ..default()
        });
    }

    // ---

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 0.0),
        ..default()
    });

    // ---

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
            Vec3::new(-10.0, 5.0, 20.0),
            Vec3::ZERO,
        ));
}
