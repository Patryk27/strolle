#[path = "_common.rs"]
mod common;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_obj::ObjPlugin;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
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
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(FpsCameraPlugin::default())
        .add_plugin(ObjPlugin)
        .add_plugin(StrollePlugin)
        .insert_resource(Sun::default())
        .add_startup_system(setup)
        .add_system(adjust_materials)
        .add_system(process_input)
        .add_system(move_light)
        .add_system(animate_sun)
        .add_system(animate_objects)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // ---

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            ..default()
        })
        .insert(StrolleCamera::default())
        .insert(FpsCameraBundle::new(
            {
                let mut controller = FpsCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.35;
                controller.translate_sensitivity = 8.0;
                controller
            },
            vec3(-5.75, 0.5, -14.5),
            vec3(-5.75, 0.5, -21.5),
            vec3(0.0, 1.0, 0.0),
        ));

    commands.spawn(SceneBundle {
        scene: assets.load("demo/level.glb#Scene0"),
        ..Default::default()
    });

    let lights = vec![
        vec3(-3.0, 0.75, -23.0),
        // vec3(-2.5, 0.75, -10.5),
        // vec3(-1.5, 0.75, 1.25),
        // vec3(-11.5, 0.75, 5.5),
        // vec3(-10.0, 0.75, 21.0),
        // vec3(-10.0, 0.75, 28.0),
        // vec3(-18.0, 0.75, -31.5),
        // vec3(-23.5, 0.75, -23.0),
        // vec3(-17.8, 0.75, -20.0),
    ];

    for light in lights {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                range: 20.0,
                radius: 0.2,
                intensity: 2500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(light),
            ..default()
        });
    }
}

/// Most of the materials seem to have too low metalicness which makes them look
/// suspicious in Strolle; let's fix that!
///
/// Arguably, a somewhat better approach would be to adjust the *.glb asset, but
/// doing this via code here is just simpler.
fn adjust_materials(mut materials: ResMut<Assets<StandardMaterial>>) {
    let suspicious_materials: Vec<_> = materials
        .iter()
        .filter_map(|(handle, material)| {
            if material.metallic != 0.25 || material.unlit {
                Some(materials.get_handle(handle))
            } else {
                None
            }
        })
        .collect();

    for handle in suspicious_materials {
        let material = materials.get_mut(&handle).unwrap();

        material.metallic = 0.25;
        material.unlit = false;
    }
}

fn process_input(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Query<(
        &mut Transform,
        &mut CameraRenderGraph,
        &mut StrolleCamera,
        &mut FpsCameraController,
    )>,
    mut light: Query<&mut PointLight>,
    mut sun: ResMut<Sun>,
) {
    let (
        camera_transform,
        mut camera_render_graph,
        mut camera,
        mut fps_camera_controller,
    ) = camera.single_mut();

    if keys.just_pressed(KeyCode::Key1) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::Image;
    }

    if keys.just_pressed(KeyCode::Key2) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DirectLightning;
    }

    if keys.just_pressed(KeyCode::Key3) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::IndirectLightning;
    }

    if keys.just_pressed(KeyCode::Key4) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::Normals;
    }

    if keys.just_pressed(KeyCode::Key5) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Key0) {
        camera_render_graph.set("core_3d");
    }

    if keys.just_pressed(KeyCode::Semicolon) {
        fps_camera_controller.enabled = !fps_camera_controller.enabled;

        let mut window = windows.single_mut();

        window.cursor.visible = fps_camera_controller.enabled;

        window.cursor.grab_mode = if fps_camera_controller.enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }

    // ---

    // TODO just for testing purposes
    if keys.just_pressed(KeyCode::Return) {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                range: 20.0,
                radius: 0.2,
                intensity: 2500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(
                camera_transform.translation,
            ),
            ..default()
        });

        // let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));

        // commands
        //     .spawn(PbrBundle {
        //         mesh: mesh.clone(),
        //         material: materials.add(StandardMaterial {
        //             base_color: Color::CRIMSON,
        //             ..default()
        //         }),
        //         ..default()
        //     })
        //     .insert(AnimatedObject {
        //         position: vec3(
        //             camera_transform.translation.x,
        //             0.1,
        //             camera_transform.translation.z,
        //         ),
        //         phase: time.elapsed_seconds(),
        //     });
    }

    // ---

    if keys.just_pressed(KeyCode::O) {
        sun.altitude -= 0.1;
    }

    if keys.just_pressed(KeyCode::P) {
        sun.altitude += 0.1;
    }

    if keys.just_pressed(KeyCode::LBracket) {
        light.single_mut().intensity /= 2.0;
    }

    if keys.just_pressed(KeyCode::RBracket) {
        light.single_mut().intensity *= 2.0;
    }
}

fn move_light(
    keys: Res<Input<KeyCode>>,
    camera: Query<&Transform, With<Camera>>,
    mut light: Query<&mut Transform, (With<PointLight>, Without<Camera>)>,
) {
    if keys.just_pressed(KeyCode::X) {
        light.single_mut().translation = camera.single().translation;
    }
}

#[derive(Resource)]
struct Sun {
    altitude: f32,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            altitude: StrolleSun::default().altitude,
        }
    }
}

fn animate_sun(
    time: Res<Time>,
    mut strolle_sun: ResMut<StrolleSun>,
    our_sun: Res<Sun>,
) {
    strolle_sun.altitude = strolle_sun.altitude
        + (our_sun.altitude - strolle_sun.altitude) * time.delta_seconds();
}

#[derive(Component)]
struct AnimatedObject {
    position: Vec3,
    phase: f32,
}

fn animate_objects(
    time: Res<Time>,
    mut objects: Query<(&mut Transform, &AnimatedObject)>,
) {
    let tt = time.elapsed_seconds();

    for (mut transform, animated) in objects.iter_mut() {
        let tt = tt + animated.phase;

        transform.translation =
            animated.position + vec3(0.0, tt.sin().abs() * 1.5, 0.0);

        transform.rotation =
            Quat::from_rotation_x(tt) * Quat::from_rotation_y(tt / 1.5);
    }
}
