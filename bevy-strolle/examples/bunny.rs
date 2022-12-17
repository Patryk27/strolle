use bevy::core_pipeline::core_3d;
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                // TODO
                width: 1280.0,
                height: 720.0,
                mode: WindowMode::Windowed,
                ..default()
            },
            ..default()
        }))
        .add_plugin(LookTransformPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(ObjPlugin)
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(toggle_raytracing)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 20.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.2, 0.2),
            reflectance: 0.0,
            perceptual_roughness: 0.0,
            ..default()
        }),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: assets.load("bunny.obj"),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            reflectance: 0.0,
            ..default()
        }),
        transform: Transform::from_scale(Vec3::splat(35.0))
            .with_translation(vec3(0.0, -1.0, 0.0)),
        ..default()
    });

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 5.0, 6.0),
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
