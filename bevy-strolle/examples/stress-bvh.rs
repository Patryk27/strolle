use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_rapier3d::prelude::*;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(512.0, 512.0),
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
            LookTransformPlugin,
            FpsCameraPlugin::default(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            StrollePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_spawning)
        .add_systems(Update, handle_despawning)
        .run();
}

fn setup(
    mut commands: Commands,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sun: ResMut<StrolleSun>,
) {
    let mut window = window.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // ---

    sun.altitude = 0.33;

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        })
        .insert(StrolleCamera {
            mode: st::CameraMode::BvhHeatmap,
        })
        .insert(FpsCameraBundle::new(
            {
                let mut controller = FpsCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.35;
                controller.translate_sensitivity = 8.0;
                controller
            },
            vec3(0.0, 8.0, 40.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
        ));

    // ---

    let box_mesh = meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0)));

    let sphere_mesh =
        meshes.add(Mesh::try_from(shape::Icosphere::default()).unwrap());

    commands
        .spawn(PbrBundle {
            mesh: box_mesh.clone(),
            material: materials.add(StandardMaterial::from(Color::WHITE)),
            transform: Transform::from_scale(vec3(100.0, 1.0, 100.0)),
            ..default()
        })
        .insert(Collider::cuboid(50.0, 0.5, 50.0));

    commands.insert_resource(State {
        box_mesh,
        sphere_mesh,
        spawner: Timer::from_seconds(0.1, TimerMode::Repeating),
    });
}

#[derive(Debug, Resource)]
struct State {
    box_mesh: Handle<Mesh>,
    sphere_mesh: Handle<Mesh>,
    spawner: Timer,
}

#[derive(Component, Debug)]
struct Object {
    despawner: Timer,
}

fn handle_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<State>,
) {
    if !state.spawner.tick(time.delta()).just_finished() {
        return;
    }

    let [rand0, rand1, rand2, rand3, rand4, ..] = time
        .elapsed_seconds_f64()
        .to_bits()
        .reflect_hash()
        .unwrap()
        .to_be_bytes();

    let mesh;
    let collider;

    if rand0 % 2 == 0 {
        mesh = state.box_mesh.clone();
        collider = Collider::cuboid(0.5, 0.5, 0.5);
    } else {
        mesh = state.sphere_mesh.clone();
        collider = Collider::ball(1.0);
    }

    let material = materials.add(StandardMaterial::from(Color::hsl(
        (rand1 as f32) / 255.0 * 360.0,
        1.0,
        0.5,
    )));

    let offset_x = (rand2 as f32) / 255.0 * 20.0 - 10.0;
    let offset_y = (rand3 as f32) / 255.0 * 20.0 - 10.0;

    commands
        .spawn(PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(vec3(
                offset_x, 20.0, offset_y,
            )),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(collider)
        .insert(Object {
            despawner: Timer::from_seconds(
                (rand4 as f32) / 255.0 * 2000.0,
                TimerMode::Once,
            ),
        });
}

fn handle_despawning(
    mut commands: Commands,
    time: Res<Time>,
    mut objects: Query<(Entity, &mut Object)>,
    keys: Res<Input<KeyCode>>,
) {
    for (object_entity, mut object) in objects.iter_mut() {
        // if object.despawner.tick(time.delta()).just_finished()
        //     || keys.just_pressed(KeyCode::X)
        // {
        //     commands.entity(object_entity).despawn();
        // }
    }
}
