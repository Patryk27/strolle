use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::uvec2;
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, Viewport};
use bevy_strolle::prelude::*;
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
        .add_plugin(LookTransformPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(update_cameras)
        .add_system(animate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        perceptual_roughness: 0.2,
        ..default()
    });

    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.2, 0.2, 0.2),
        perceptual_roughness: 0.0,
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: floor_mat,
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat.clone(),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Animated { phase: 0.0 });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: cube_mat,
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

    // -----

    let transform = Transform::from_xyz(15.0, 10.0, 15.0)
        .looking_at(Vec3::splat(0.0), Vec3::Y);

    let bevy_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                priority: 0,
                ..default()
            },
            transform,
            ..default()
        })
        .id();

    let strolle_image_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                priority: 1,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .id();

    let strolle_normals_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                priority: 2,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .insert(StrolleCamera {
            config: st::ViewportConfiguration {
                mode: st::ViewportMode::DisplayNormals,
                ..default()
            },
        })
        .id();

    let strolle_bvh_heatmap_camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                priority: 3,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            transform,
            ..default()
        })
        .insert(StrolleCamera {
            config: st::ViewportConfiguration {
                mode: st::ViewportMode::DisplayBvhHeatmap,
                ..default()
            },
        })
        .id();

    // -----

    commands.insert_resource(State {
        cameras: [
            bevy_camera,
            strolle_image_camera,
            strolle_normals_camera,
            strolle_bvh_heatmap_camera,
        ],
    });
}

const RADIUS: f32 = 3.0;

#[derive(Resource)]
struct State {
    cameras: [Entity; 4],
}

fn update_cameras(
    state: Res<State>,
    windows: Res<Windows>,
    mut cameras: Query<&mut Camera>,
) {
    let window = windows.primary();

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

fn animate(time: Res<Time>, mut objects: Query<(&mut Transform, &Animated)>) {
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
