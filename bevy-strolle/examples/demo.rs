#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;
use bevy::asset::AssetContainer;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2, vec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget};
use bevy::render::renderer::RenderDevice;
use bevy::render::RenderPlugin;
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::render::texture::ImageSampler;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use wgpu::{Extent3d, Limits, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy_strolle::graph::StrolleGraph;
use self::common::Sun;

const VIEWPORT_SIZE: UVec2 = uvec2(640, 480);
const WINDOW_SCALE: f32 = 1.5;

fn main() {
    common::extract_assets();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(
                        WINDOW_SCALE * VIEWPORT_SIZE.x as f32,
                        WINDOW_SCALE * VIEWPORT_SIZE.y as f32,
                    ),
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            StrollePlugin,
        ))
        .add_systems(Startup, setup_window)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, check_gpu_limits)
        .add_systems(Update, adjust_materials)
        .add_systems(Update, handle_materials)
        .add_systems(Update, common::handle_camera)
        .add_systems(Update, common::handle_sun)
        .add_systems(Update, common::animate_sun)
        .add_systems(Update, handle_flashlight)
        .add_systems(Update, animate_flashlight)
        .add_systems(Update, animate_toruses)
        .insert_resource(Sun::default())
        .run();
}

fn check_gpu_limits(render_device: Res<RenderDevice>) {
    let limits = render_device.limits();
    info!("GPU Limits:");
    info!("  max_color_attachment_bytes_per_sample: {}",
          limits.max_color_attachment_bytes_per_sample);
    info!("  max_texture_dimension_2d: {}",
          limits.max_texture_dimension_2d);
    info!("  max_storage_buffer_binding_size: {}",
          limits.max_storage_buffer_binding_size);
    info!("  max_bind_groups: {}",
          limits.max_bind_groups);
    info!("  max_bindings_per_bind_group: {}",
          limits.max_bindings_per_bind_group);
}

fn setup_window(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn setup_camera(
    mut commands: Commands,
    mut window: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = window.single_mut();

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
        sampler: ImageSampler::nearest(),
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
        camera_2d: Camera2d {},
        ..default()
    });

    commands.spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                StrolleGraph,
            ),
            camera: Camera {
                order: 0,
                target: RenderTarget::Image(viewport),
                hdr: true,
                ..default()
            },
            ..default()
        }).insert(StrolleCamera::default())
        .insert(FpsCameraBundle::new(
            {
                FpsCameraController {
                    enabled: true,
                    mouse_rotate_sensitivity: Vec2::ONE * 0.35,
                    translate_sensitivity: 8.0,
                    ..default()
                }
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

    // ---

    let lights = vec![
        vec3(-3.0, 0.75, -23.0),
        vec3(-23.5, 0.75, -31.0),
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
                intensity: 1000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(light),
            ..default()
        });
    }

    // ---

    let toruses = [
        vec3(-0.5, 0.33, -5.5),
        vec3(-11.0, 0.33, 28.0),
        vec3(-11.5, 0.33, 13.5),
    ];

    for torus in toruses {
        let color = Color::rgba(0.9, 0.6, 0.3, 1.0);

        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(Torus::default())),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.to_linear(),
                    ..default()
                }),
                transform: Transform::from_translation(torus)
                    .with_rotation(Quat::from_rotation_z(1.0))
                    .with_scale(Vec3::splat(0.5)),
                ..default()
            })
            .insert(StrTorus);
    }

    // ---

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
    keys: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        for (_, material) in materials.iter_mut() {
            material.base_color_texture = None;
        }
    }

    if keys.just_pressed(KeyCode::KeyM) {
        for (_, material) in materials.iter_mut() {
            if material.metallic == 0.0 {
                material.metallic = 1.0;
                material.perceptual_roughness = 0.15;
            } else {
                material.metallic = 0.0;
                material.perceptual_roughness = 1.0;
            }
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Component)]
struct Flashlight {
    enabled: bool,
}

fn handle_flashlight(
    keys: Res<ButtonInput<KeyCode>>,
    mut flashlight: Query<(&mut Flashlight, &mut SpotLight)>,
    mut lights: Query<&mut PointLight>,
) {
    let (mut flashlight, mut flashlight_spot) = flashlight.single_mut();

    if keys.just_pressed(KeyCode::KeyF) {
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

// -----------------------------------------------------------------------------

#[derive(Component)]
struct StrTorus;

fn animate_toruses(
    time: Res<Time>,
    mut toruses: Query<&mut Transform, (With<StrTorus>, Without<Flashlight>)>,
) {
    for mut xform in toruses.iter_mut() {
        xform.rotation = Quat::from_rotation_z(time.elapsed_seconds())
            * Quat::from_rotation_x(time.elapsed_seconds() + 1.0);
    }
}
