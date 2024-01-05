#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use strolle::StrollePlugin;

fn main() {
    common::unzip_assets();

    App::new()
        .add_plugins((
            DefaultPlugins,
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            StrollePlugin,
        ))
        .add_systems(Startup, setup_window)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, adjust_materials)
        .add_systems(Update, handle_materials)
        .add_systems(Update, handle_camera)
        .add_systems(Update, handle_sun)
        .add_systems(Update, animate_sun)
        .add_systems(Update, handle_flashlight)
        .add_systems(Update, animate_flashlight)
        .insert_resource(Sun::default())
        .run();
}

fn setup_window(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                strolle::graph::BVH_HEATMAP,
            ),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        })
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
}

fn setup_scene(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(SceneBundle {
        scene: assets.load("demo/level.glb#Scene0"),
        ..default()
    });

    let lights = vec![
        vec3(-3.0, 0.75, -23.0),
        vec3(-17.5, 0.75, -31.0),
        vec3(-23.75, 0.75, -24.0),
        vec3(1.25, 0.75, -10.5),
        vec3(-3.15, 0.75, 1.25),
        vec3(-3.25, 0.75, 20.25),
        vec3(13.25, 0.75, -28.25),
    ];

    for light in lights {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                range: 35.0,
                radius: 0.15,
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(light),
            ..default()
        });
    }

    let cubes = [
        vec3(-0.5, 0.33, -5.5),
        vec3(-11.0, 0.33, 28.0),
        vec3(-11.5, 0.33, 13.5),
    ];

    for cube in cubes {
        let color = Color::rgba(0.85, 0.05, 0.25, 1.0);

        commands.spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Cube::new(0.33))),
            material: materials.add(StandardMaterial {
                base_color: color,
                emissive: color * 5.0,
                ..default()
            }),
            transform: Transform::from_translation(cube),
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

// -----------------------------------------------------------------------------

/// Most of materials in our demo-scene have the `unlit` flag toggled on, making
/// it look suspicious - let's fix that!
///
/// Arguably, a somewhat better approach would be to adjust the *.glb asset, but
/// doing this via code here is just simpler.
fn adjust_materials(
    mut materials_adjusted: Local<bool>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if *materials_adjusted || materials.len() < 16 {
        return;
    }

    for (_, material) in materials.iter_mut() {
        material.unlit = false;
        material.reflectance = 0.0;
        material.perceptual_roughness = 1.0;
    }

    *materials_adjusted = true;
}

fn handle_materials(
    keys: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::T) {
        for (_, material) in materials.iter_mut() {
            material.base_color_texture = None;
        }
    }
}

// -----------------------------------------------------------------------------

fn handle_camera(
    keys: Res<Input<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<(
        &Transform,
        &mut CameraRenderGraph,
        &mut FpsCameraController,
    )>,
) {
    let (camera_xform, mut camera_graph, mut camera_ctrl) = camera.single_mut();

    if keys.just_pressed(KeyCode::Key1) {
        camera_graph.set(strolle::graph::BVH_HEATMAP);
    }

    if keys.just_pressed(KeyCode::Key0) {
        camera_graph.set("core_3d");
    }

    if keys.just_pressed(KeyCode::Semicolon) {
        camera_ctrl.enabled = !camera_ctrl.enabled;

        let mut window = window.single_mut();

        window.cursor.visible = !camera_ctrl.enabled;

        window.cursor.grab_mode = if camera_ctrl.enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }

    if keys.just_pressed(KeyCode::X) {
        println!("{:?}", camera_xform.translation);
    }
}

// -----------------------------------------------------------------------------

#[derive(Resource)]
struct Sun {
    azimuth: f32,
    altitude: f32,
    initialized: bool,
}

impl Default for Sun {
    fn default() -> Self {
        Self {
            azimuth: 3.0,
            altitude: strolle::Sun::default().altitude,
            initialized: false,
        }
    }
}

fn handle_sun(keys: Res<Input<KeyCode>>, mut sun: ResMut<Sun>) {
    if keys.just_pressed(KeyCode::H) {
        sun.azimuth -= 0.05;
    }

    if keys.just_pressed(KeyCode::J) {
        sun.altitude -= 0.05;
    }

    if keys.just_pressed(KeyCode::K) {
        sun.altitude += 0.05;
    }

    if keys.just_pressed(KeyCode::L) {
        sun.azimuth += 0.05;
    }
}

fn animate_sun(
    time: Res<Time>,
    mut st_sun: ResMut<strolle::Sun>,
    mut sun: ResMut<Sun>,
) {
    if sun.initialized {
        st_sun.azimuth = st_sun.azimuth
            + (sun.azimuth - st_sun.azimuth) * time.delta_seconds();

        st_sun.altitude = st_sun.altitude
            + (sun.altitude - st_sun.altitude) * time.delta_seconds();
    } else {
        sun.initialized = true;
        st_sun.azimuth = sun.azimuth;
        st_sun.altitude = sun.altitude;
    }
}

// -----------------------------------------------------------------------------

#[derive(Component)]
struct Flashlight {
    enabled: bool,
}

fn handle_flashlight(
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
