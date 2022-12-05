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
        .add_system(switch_camera_render_graphs)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(OrbitCameraBundle::new(
            {
                let mut controller = OrbitCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.2;
                controller.mouse_translate_sensitivity = Vec2::ONE * 0.5;
                controller
            },
            Vec3::new(-20.0, 10.0, 20.0),
            Vec3::ZERO,
        ));
}

pub fn switch_camera_render_graphs(
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
