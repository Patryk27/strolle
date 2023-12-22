use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseWheel;
use bevy::math::{uvec2, vec2, vec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget};
use bevy::render::texture::ImageSampler;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_mod_raycast::prelude::*;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

const VIEWPORT_SIZE: UVec2 = uvec2(800, 600);
const WINDOW_SCALE: f32 = 1.0;

fn main() {
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
            FrameTimeDiagnosticsPlugin::default(),
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            StrollePlugin,
        ))
        .add_systems(Startup, setup_window)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_state)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, handle_camera)
        .add_systems(Update, process_input)
        .add_systems(Update, process_blocks.after(process_input))
        .add_systems(Update, update_crosshair.after(process_input))
        // ---
        .add_systems(Update, handle_sun)
        .add_systems(Update, animate_sun)
        .insert_resource(Sun::default())
        // ---
        .add_event::<Event>()
        .run();
}

fn setup_window(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Confined;
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
                hdr: true,
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
            vec3(2.0, 1.0, 2.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ));
}

fn setup_state(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let block_mesh = meshes.add(Mesh::from(shape::Cube::new(1.0)));
    let block_white_mat = materials.add(StandardMaterial::from(Color::WHITE));
    let block_red_mat = materials.add(StandardMaterial::from(Color::RED));
    let block_green_mat = materials.add(StandardMaterial::from(Color::GREEN));
    let block_blue_mat = materials.add(StandardMaterial::from(Color::BLUE));

    let torch_mesh = meshes.add(Mesh::from(shape::Cylinder {
        radius: 0.1,
        height: 0.4,
        resolution: 16,
        segments: 1,
    }));

    let torch_mat = materials.add(StandardMaterial::from(Color::YELLOW));

    let items = vec![
        Item::Block {
            mesh: block_mesh.clone(),
            material: block_white_mat.clone(),
            crosshair: Color::WHITE,
        },
        Item::Block {
            mesh: block_mesh.clone(),
            material: block_red_mat.clone(),
            crosshair: Color::RED,
        },
        Item::Block {
            mesh: block_mesh.clone(),
            material: block_green_mat.clone(),
            crosshair: Color::GREEN,
        },
        Item::Block {
            mesh: block_mesh.clone(),
            material: block_blue_mat.clone(),
            crosshair: Color::BLUE,
        },
        Item::Torch {
            mesh: torch_mesh.clone(),
            material: torch_mat.clone(),
            crosshair: Color::YELLOW,
        },
    ];

    // ---

    let crosshair_mesh = meshes.add(shape::Box::new(2.0, 16.0, 0.0).into());

    let crosshair_material =
        color_materials.add(ColorMaterial::from(Color::WHITE));

    commands.spawn(MaterialMesh2dBundle {
        mesh: crosshair_mesh.clone().into(),
        material: crosshair_material.clone().into(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        ..default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: crosshair_mesh.into(),
        material: crosshair_material.clone().into(),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0))
            .with_rotation(Quat::from_rotation_z(PI / 2.0)),
        ..default()
    });

    // ---

    commands.insert_resource(State {
        items,
        selected_item: 0,
        crosshair_material,
    });
}

fn setup_scene(mut events: EventWriter<Event>) {
    events.send(Event::Spawn {
        position: vec3(0.0, 0.0, 0.0),
        item: 0,
    });
}

fn handle_camera(
    keys: Res<Input<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<(
        &mut CameraRenderGraph,
        &mut StrolleCamera,
        &mut FpsCameraController,
    )>,
) {
    let (mut camera_render_graph, mut camera, mut fps_camera_controller) =
        camera.single_mut();

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
        camera.mode = st::CameraMode::IndirectDiffuseLightning;
    }

    if keys.just_pressed(KeyCode::Key4) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::IndirectSpecularLightning;
    }

    if keys.just_pressed(KeyCode::Key5) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Key9) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::Reference { depth: 1 };
    }

    if keys.just_pressed(KeyCode::Key0) {
        camera_render_graph.set("core_3d");
    }

    if keys.just_pressed(KeyCode::Semicolon) {
        fps_camera_controller.enabled = !fps_camera_controller.enabled;

        let mut window = window.single_mut();

        window.cursor.visible = !fps_camera_controller.enabled;

        window.cursor.grab_mode = if fps_camera_controller.enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

fn process_input(
    mut raycast: Raycast,
    mut state: ResMut<State>,
    mouse: Res<Input<MouseButton>>,
    mut wheel: EventReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<StrolleCamera>>,
    objects: Query<Entity, With<Object>>,
    mut events: EventWriter<Event>,
) {
    let window = window.single();
    let (camera, camera_xform) = camera.single();

    // ---

    if mouse.just_pressed(MouseButton::Left)
        || mouse.just_pressed(MouseButton::Middle)
        || mouse.just_pressed(MouseButton::Right)
    {
        let ray = camera.viewport_to_world(
            camera_xform,
            vec2(window.width(), window.height()) / WINDOW_SCALE / 2.0,
        );

        if let Some(ray) = ray {
            let intersections = {
                let ray = Ray3d::new(ray.origin, ray.direction);
                let filter = |entity| objects.contains(entity);
                let settings = RaycastSettings::default().with_filter(&filter);

                raycast.cast_ray(ray, &settings)
            };

            if let Some((intersected, intersection)) = intersections.first() {
                let cell =
                    intersection.position() - 0.5 * intersection.normal();

                let cell = cell.round();

                if mouse.just_pressed(MouseButton::Left) {
                    events.send(Event::Destroy {
                        entity: *intersected,
                    });
                } else if mouse.just_pressed(MouseButton::Middle) {
                    events.send(Event::Destroy {
                        entity: *intersected,
                    });

                    events.send(Event::Spawn {
                        position: cell,
                        item: state.selected_item,
                    });
                } else {
                    events.send(Event::Spawn {
                        position: cell.round() + intersection.normal(),
                        item: state.selected_item,
                    });
                }
            }
        }
    }

    // ---

    for event in wheel.iter() {
        if event.y < 0.0 {
            state.selected_item = state
                .selected_item
                .checked_sub(1)
                .unwrap_or_else(|| state.items.len() - 1);
        } else if event.y > 0.0 {
            state.selected_item = (state.selected_item + 1) % state.items.len();
        }
    }
}

fn process_blocks(
    state: Res<State>,
    mut commands: Commands,
    mut events: EventReader<Event>,
) {
    for block in events.iter() {
        match block {
            Event::Spawn { position, item } => {
                let item = &state.items[*item];

                match item {
                    Item::Block { mesh, material, .. } => {
                        commands
                            .spawn(MaterialMeshBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_translation(
                                    *position,
                                ),
                                ..default()
                            })
                            .insert(Object);
                    }

                    Item::Torch { mesh, material, .. } => {
                        commands
                            .spawn(MaterialMeshBundle {
                                mesh: mesh.clone(),
                                material: material.clone(),
                                transform: Transform::from_translation(
                                    *position - vec3(0.0, 0.3, 0.0),
                                ),
                                ..default()
                            })
                            .insert(Object)
                            .with_children(|commands| {
                                commands
                                    .spawn(PointLightBundle {
                                        point_light: PointLight {
                                            color: Color::WHITE,
                                            range: 35.0,
                                            radius: 0.1,
                                            intensity: 250.0,
                                            shadows_enabled: true,
                                            ..default()
                                        },
                                        transform: Transform::from_translation(
                                            vec3(0.0, 0.75, 0.0),
                                        ),
                                        ..default()
                                    })
                                    .insert(Object);
                            });
                    }
                }
            }

            Event::Destroy { entity } => {
                commands.entity(*entity).despawn_recursive();
            }
        }
    }
}

fn update_crosshair(
    state: Res<State>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    materials.get_mut(&state.crosshair_material).unwrap().color =
        state.items[state.selected_item].crosshair();
}

#[derive(Debug, Resource)]
struct State {
    items: Vec<Item>,
    selected_item: usize,
    crosshair_material: Handle<ColorMaterial>,
}

#[derive(Debug, Event)]
enum Event {
    Spawn { position: Vec3, item: usize },
    Destroy { entity: Entity },
}

#[derive(Debug)]
enum Item {
    Block {
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        crosshair: Color,
    },
    Torch {
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        crosshair: Color,
    },
}

impl Item {
    pub fn crosshair(&self) -> Color {
        match self {
            Item::Block { crosshair, .. } | Item::Torch { crosshair, .. } => {
                *crosshair
            }
        }
    }
}

#[derive(Component)]
struct Object;

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
            altitude: StrolleSun::default().altitude,
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
    mut st_sun: ResMut<StrolleSun>,
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
