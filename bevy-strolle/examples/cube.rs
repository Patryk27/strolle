use bevy::core_pipeline::core_3d;
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
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(animate)
        .add_system(toggle_raytracing)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        })
        .insert(Animated);

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
        .spawn(Camera3dBundle::default())
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

fn animate(
    time: Res<Time>,
    mut objects: Query<&mut Transform, With<Animated>>,
) {
    let tt = time.elapsed_seconds();

    for mut object in objects.iter_mut() {
        object.translation.y = 0.5 + tt.sin().abs() * 1.8;

        object.rotation =
            Quat::from_rotation_x(tt) * Quat::from_rotation_y(tt / 1.5);
    }
}

fn toggle_raytracing(
    mut camera: Query<&mut CameraRenderGraph>,
    keys: Res<Input<KeyCode>>,
) {
    let default_render_graph = CameraRenderGraph::new(core_3d::graph::NAME);

    let strolle_render_graph =
        CameraRenderGraph::new(bevy_strolle::graph::NAME);

    if keys.just_pressed(KeyCode::G) {
        let mut camera = camera.single_mut();

        if camera.as_ref().as_ref() == default_render_graph.as_ref() {
            *camera = strolle_render_graph;
        } else {
            *camera = default_render_graph;
        }
    }
}

#[derive(Component)]
struct Animated;
