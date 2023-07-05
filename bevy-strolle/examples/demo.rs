#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget};
use bevy::render::texture::ImageSampler;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_obj::ObjPlugin;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

const VIEWPORT_SIZE: UVec2 = uvec2(640, 480);

fn main() {
    common::unzip_assets();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    1.5 * VIEWPORT_SIZE.x as f32,
                    1.5 * VIEWPORT_SIZE.y as f32,
                ),
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
        .add_system(handle_camera)
        .add_system(handle_sun)
        .add_system(animate_sun)
        .add_system(handle_flashlight)
        .add_system(animate_flashlight)
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut window = windows.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // ---

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

    // -------------------------------------------------------------------------

    let mut viewport = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: VIEWPORT_SIZE.x,
                height: VIEWPORT_SIZE.y,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        sampler_descriptor: ImageSampler::nearest(),
        ..default()
    };

    viewport.resize(Extent3d {
        width: VIEWPORT_SIZE.x,
        height: VIEWPORT_SIZE.y,
        depth_or_array_layers: 1,
    });

    let viewport = images.add(viewport);

    commands.spawn(SpriteBundle {
        texture: viewport.clone(),
        transform: Transform::from_scale(vec3(
            window.width() / (VIEWPORT_SIZE.x as f32),
            window.height() / (VIEWPORT_SIZE.y as f32),
            1.0,
        )),
        ..default()
    });

    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::None,
        },
        ..default()
    });

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            camera: Camera {
                order: 0,
                target: RenderTarget::Image(viewport),
                ..default()
            },
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

// -----------------------------------------------------------------------------

fn handle_camera(
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
        camera.mode = st::CameraMode::DemodulatedDirectLightning;
    }

    if keys.just_pressed(KeyCode::Key4) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::IndirectLightning;
    }

    if keys.just_pressed(KeyCode::Key5) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DemodulatedIndirectLightning;
    }

    if keys.just_pressed(KeyCode::Key6) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::NormalMap;
    }

    if keys.just_pressed(KeyCode::Key7) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Key8) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::VelocityMap;
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

fn handle_sun(keys: Res<Input<KeyCode>>, mut sun: ResMut<Sun>) {
    if keys.just_pressed(KeyCode::O) {
        sun.altitude -= 0.05;
    }

    if keys.just_pressed(KeyCode::P) {
        sun.altitude += 0.05;
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
    camera: Query<&Transform, With<StrolleCamera>>,
    mut flashlight: Query<
        &mut Transform,
        (With<Flashlight>, Without<StrolleCamera>),
    >,
) {
    let camera = camera.single();
    let mut flashlight = flashlight.single_mut();

    *flashlight =
        Transform::from_translation(camera.translation - vec3(0.0, 0.25, 0.0))
            .with_rotation(camera.rotation);
}
