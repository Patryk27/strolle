#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

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
        .add_system(process_input_camera)
        .add_system(process_input_flashlight)
        .add_system(process_input_sun)
        .add_system(animate_sun)
        .add_system(animate_flashlight)
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
            vec3(-5.75, 0.5, -16.8),
            vec3(-5.75, 0.5, -17.0),
            vec3(0.0, 1.0, 0.0),
        ));

    commands.spawn(SceneBundle {
        scene: assets.load("demo/level.glb#Scene0"),
        ..Default::default()
    });

    let lights = vec![
        vec3(-3.0, 0.75, -23.0),
        vec3(-17.5, 0.75, -31.0),
        vec3(-23.75, 0.75, -24.0),
        vec3(1.25, 0.75, -10.5),
        vec3(-3.15, 0.75, 1.25),
        vec3(-3.25, 0.75, 20.25),
        vec3(-11.5, 0.75, 28.50),
        vec3(13.25, 0.75, -28.25),
        vec3(1.15, 0.75, -3.75),
    ];

    for light in lights {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                range: 35.0,
                radius: 0.25,
                intensity: 6000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(light),
            ..default()
        });
    }

    commands
        .spawn(SpotLightBundle {
            spot_light: SpotLight {
                color: Color::WHITE,
                range: 100.0,
                radius: 0.1,
                intensity: 0.0,
                shadows_enabled: true,
                inner_angle: 0.1 * PI,
                outer_angle: 0.1 * PI,
                ..default()
            },
            ..default()
        })
        .insert(Flashlight { enabled: false });
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

fn process_input_camera(
    keys: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<(
        &Transform,
        &mut CameraRenderGraph,
        &mut StrolleCamera,
        &mut FpsCameraController,
    )>,
) {
    let (
        camera_xform,
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
        camera.mode = st::CameraMode::DemodulatedIndirectLightning;
    }

    if keys.just_pressed(KeyCode::Key5) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::Normals;
    }

    if keys.just_pressed(KeyCode::Key6) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Key0) {
        camera_render_graph.set("core_3d");
    }

    if keys.just_pressed(KeyCode::Semicolon) {
        fps_camera_controller.enabled = !fps_camera_controller.enabled;

        let mut window = windows.single_mut();

        window.cursor.visible = !fps_camera_controller.enabled;

        window.cursor.grab_mode = if fps_camera_controller.enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }

    if keys.just_pressed(KeyCode::T) {
        println!("{:?}", camera_xform.translation);
    }
}

fn process_input_flashlight(
    keys: Res<Input<KeyCode>>,
    mut flashlight: Query<(&mut Flashlight, &mut SpotLight)>,
    mut lights: Query<&mut PointLight>,
) {
    let (mut flashlight, mut flashlight_spot) = flashlight.single_mut();

    if keys.just_pressed(KeyCode::F) {
        flashlight.enabled = !flashlight.enabled;

        if flashlight.enabled {
            flashlight_spot.intensity = 16000.0;
        } else {
            flashlight_spot.intensity = 0.0;
        }

        for mut light in lights.iter_mut() {
            light.intensity = if flashlight.enabled { 0.0 } else { 6000.0 };
        }
    }
}

fn process_input_sun(keys: Res<Input<KeyCode>>, mut sun: ResMut<Sun>) {
    if keys.just_pressed(KeyCode::O) {
        sun.altitude -= 0.05;
    }

    if keys.just_pressed(KeyCode::P) {
        sun.altitude += 0.05;
    }
}

// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------

#[derive(Component)]
struct Flashlight {
    enabled: bool,
}

fn animate_flashlight(
    camera: Query<&Transform, With<Camera>>,
    mut flashlight: Query<&mut Transform, (With<Flashlight>, Without<Camera>)>,
) {
    let camera = camera.single();
    let mut flashlight = flashlight.single_mut();

    *flashlight =
        Transform::from_translation(camera.translation - vec3(0.0, 0.25, 0.0))
            .with_rotation(camera.rotation);
}
