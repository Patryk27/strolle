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
        .add_startup_system(setup)
        .add_system(fixup_scene)
        .add_system(handle_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut sun: ResMut<StrolleSun>,
) {
    let mut window = windows.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // -------------------------------------------------------------------------

    commands.spawn(SceneBundle {
        // scene: assets.load("/Users/PWY/Desktop/cube.gltf#Scene0"),
        scene: assets.load(
            "/Users/PWY/Downloads/scenes/cartoon_lowpoly_small_city_free_pack.glb#Scene0",
        ),
        // scene: assets.load(
        //     "/Users/PWY/Downloads/free__atlanta_corperate_office_building.glb#Scene0",
        // ),
        transform: Transform::from_scale(Vec3::splat(0.1)),
        ..Default::default()
    });

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
        .insert(StrolleCamera {
            mode: st::CameraMode::Image,
        })
        .insert(FpsCameraBundle::new(
            {
                let mut controller = FpsCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.35;
                controller.translate_sensitivity = 8.0;
                controller
            },
            vec3(7.5485477, 7.116934, -5.4978814),
            vec3(6.8574367, 6.5966244, -4.9962406),
            vec3(0.0, 1.0, 0.0),
        ));

    // -------------------------------------------------------------------------

    // sun.altitude = 0.05;
}

fn handle_camera(
    mut commands: Commands,
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
        camera.mode = st::CameraMode::Reference { depth: 1 };
    }

    if keys.just_pressed(KeyCode::Key3) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::Reference { depth: 3 };
    }

    if keys.just_pressed(KeyCode::Key4) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Key5) {
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

    if keys.just_pressed(KeyCode::X) {
        println!("{:?}", camera_xform.translation);
        println!("{:?}", camera_xform.translation + camera_xform.forward());
    }

    if keys.just_pressed(KeyCode::T) {
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                intensity: 6000.0,
                radius: 0.25,
                range: 35.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(camera_xform.translation),
            ..default()
        });
    }
}

fn fixup_scene(mut done: Local<bool>, mut lights: Query<&mut PointLight>) {
    if *done || lights.is_empty() {
        return;
    }

    for mut light in lights.iter_mut() {
        light.radius = 0.33;
        light.intensity *= 0.005;
        light.shadows_enabled = true;
    }

    *done = true;
}
