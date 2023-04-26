#[path = "_common.rs"]
mod common;

use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::utils::HashMap;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowResolution};
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::{
    FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

const MAP: &str = r#"
    ###   ###
    #.#   #B#
    #.# ###.#
    #.# #....####
    #.###.....#.##
    #.......'..R.#
    #.###.....#.##
    #.# #....##*#
    #.#####.#.#.#
    #........G###
    ###########
"#;

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
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(process_input)
        .run();
}

fn setup(
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut window = windows.single_mut();

    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;

    // ---

    let spawnpoint = spawn_map(
        LevelBuilder::new(
            &mut commands,
            &mut asset_server,
            &mut meshes,
            &mut materials,
        ),
        MAP,
    );

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
            vec3(0.0, 1.0, 3.2) + spawnpoint,
            vec3(0.0, 1.0, 0.0) + spawnpoint,
            vec3(0.0, 1.0, 0.0),
        ));
}

fn process_input(
    keys: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<(
        &mut CameraRenderGraph,
        &mut StrolleCamera,
        &mut FpsCameraController,
    )>,
    mut light: Query<&mut PointLight>,
) {
    let (mut camera_render_graph, mut camera, mut fps_camera_controller) =
        camera.single_mut();

    if keys.just_pressed(KeyCode::Key1) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DisplayImage;
    }

    if keys.just_pressed(KeyCode::Key2) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DisplayDirectLightning;
    }

    if keys.just_pressed(KeyCode::Key3) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DisplayIndirectLightning;
    }

    if keys.just_pressed(KeyCode::Key4) {
        camera_render_graph.set(bevy_strolle::graph::NAME);
        camera.mode = st::CameraMode::DisplayNormals;
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

    if keys.just_pressed(KeyCode::LBracket) {
        light.single_mut().intensity *= 2.0;
    }

    if keys.just_pressed(KeyCode::RBracket) {
        light.single_mut().intensity /= 2.0;
    }

    if keys.just_pressed(KeyCode::Z) {
        light.single_mut().color = Color::WHITE;
    }

    if keys.just_pressed(KeyCode::X) {
        light.single_mut().color = Color::VIOLET;
    }
}

struct LevelBuilder<'a, 'w, 's> {
    commands: &'a mut Commands<'w, 's>,
    materials: &'a mut Assets<StandardMaterial>,

    floor_mesh: Handle<Mesh>,
    floor_material: Handle<StandardMaterial>,

    ceil_mesh: Handle<Mesh>,
    ceil_material: Handle<StandardMaterial>,

    wall_mesh: Handle<Mesh>,
    wall_material: Handle<StandardMaterial>,

    sphere_mesh: Handle<Mesh>,
}

impl<'a, 'w, 's> LevelBuilder<'a, 'w, 's> {
    fn new(
        commands: &'a mut Commands<'w, 's>,
        asset_server: &'a mut AssetServer,
        meshes: &'a mut Assets<Mesh>,
        materials: &'a mut Assets<StandardMaterial>,
    ) -> Self {
        Self {
            floor_mesh: meshes.add(Mesh::from(shape::Plane {
                size: 3.0,
                subdivisions: 0,
            })),

            floor_material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                base_color_texture: Some(
                    asset_server.load("dungeon/floor.diffuse.png"),
                ),
                // TODO
                // normal_map_texture: Some(
                //     asset_server.load("dungeon/floor.normal.png"),
                // ),
                reflectance: 0.0,
                metallic: 0.05,
                ..default()
            }),

            ceil_mesh: meshes.add(Mesh::from(shape::Plane {
                size: 1.0,
                subdivisions: 0,
            })),

            ceil_material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                base_color_texture: Some(
                    asset_server.load("dungeon/floor.diffuse.png"),
                ),
                // TODO
                // normal_map_texture: Some(
                //     asset_server.load("dungeon/floor.normal.png"),
                // ),
                reflectance: 0.0,
                ..default()
            }),

            wall_mesh: meshes.add({
                let mut mesh = Mesh::from(shape::Plane {
                    size: 1.0,
                    subdivisions: 0,
                });

                mesh.generate_tangents().unwrap();
                mesh
            }),

            wall_material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 1.0),
                base_color_texture: Some(
                    asset_server.load("dungeon/wall.diffuse.jpg"),
                ),
                normal_map_texture: Some(
                    asset_server.load("dungeon/wall.normal.png"),
                ),
                reflectance: 0.0,
                metallic: 0.2,
                ..default()
            }),

            sphere_mesh: meshes.add(
                Mesh::try_from(shape::Icosphere {
                    radius: 0.75,
                    subdivisions: 4,
                })
                .unwrap(),
            ),

            commands,
            materials,
        }
    }

    fn position(&self, x: usize, z: usize) -> (f32, f32) {
        (3.0 * (x as f32), 3.0 * (z as f32))
    }

    fn floor(&mut self, x: usize, z: usize) {
        let (x, z) = self.position(x, z);
        let transform = Transform::from_translation(vec3(x, 0.0, z));

        self.commands.spawn(PbrBundle {
            mesh: self.floor_mesh.clone(),
            material: self.floor_material.clone(),
            transform,
            ..default()
        });
    }

    fn ceil(&mut self, x: usize, z: usize) {
        let (x, z) = self.position(x, z);

        let transform = Transform::from_translation(vec3(x, 2.4, z))
            .with_rotation(Quat::from_rotation_x(PI))
            .with_scale(vec3(3.0, 1.0, 3.0));

        self.commands.spawn(PbrBundle {
            mesh: self.ceil_mesh.clone(),
            material: self.ceil_material.clone(),
            transform,
            ..default()
        });
    }

    fn wall(&mut self, x: usize, z: usize, rot: f32) {
        let (x, z) = self.position(x, z);

        let mut transform = Transform::from_translation(vec3(x, 2.5 / 2.0, z))
            .with_rotation(
                Quat::from_rotation_x(PI / 2.0) * Quat::from_rotation_z(rot),
            )
            .with_scale(vec3(3.0, 1.0, 2.5));

        transform.translation.x -= rot.sin() * 1.5;
        transform.translation.z += rot.cos() * 1.5;

        self.commands.spawn(PbrBundle {
            mesh: self.wall_mesh.clone(),
            material: self.wall_material.clone(),
            transform,
            ..default()
        });
    }

    fn sphere(&mut self, x: usize, z: usize, tile: SphereTile) {
        let (x, z) = self.position(x, z);

        let material = self.materials.add(StandardMaterial {
            base_color: tile.color(),
            metallic: 0.25,
            ..default()
        });

        let transform = Transform::from_translation(vec3(x, 0.75, z));

        self.commands.spawn(PbrBundle {
            mesh: self.sphere_mesh.clone(),
            material,
            transform,
            ..default()
        });
    }

    fn light(&mut self, x: usize, z: usize) {
        let (x, z) = self.position(x, z);

        self.commands.spawn(PointLightBundle {
            point_light: PointLight {
                color: Color::WHITE,
                intensity: 1600.0,
                range: 50.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(vec3(x, 0.95, z)),
            ..default()
        });
    }
}

fn spawn_map(mut lvlb: LevelBuilder, map: &str) -> Vec3 {
    let map = parse_map(map);
    let mut spawnpoint = None;

    for (&(x, z), tile) in &map {
        match tile {
            Tile::Floor => {
                lvlb.floor(x, z);
                lvlb.ceil(x, z);
            }

            Tile::Wall => {
                // Handled separately below
            }

            Tile::Light => {
                lvlb.floor(x, z);
                lvlb.ceil(x, z);
                lvlb.light(x, z);
            }

            Tile::Spawnpoint => {
                lvlb.floor(x, z);
                lvlb.ceil(x, z);

                spawnpoint = Some(lvlb.position(x, z));
            }

            Tile::Sphere(tile) => {
                lvlb.floor(x, z);
                lvlb.ceil(x, z);
                lvlb.sphere(x, z, *tile);
            }
        }

        if tile.is_floorlike() {
            let is_wall_nearby = |dx: isize, dz: isize| -> bool {
                let x = x.checked_add_signed(dx);
                let z = z.checked_add_signed(dz);

                if let (Some(x), Some(z)) = (x, z) {
                    map.get(&(x, z)).map_or(false, |tile| tile.is_wall())
                } else {
                    false
                }
            };

            if is_wall_nearby(0, -1) {
                lvlb.wall(x, z - 1, 0.0 * PI);
            }

            if is_wall_nearby(1, 0) {
                lvlb.wall(x + 1, z, 0.5 * PI);
            }

            if is_wall_nearby(0, 1) {
                lvlb.wall(x, z + 1, 1.0 * PI);
            }

            if is_wall_nearby(-1, 0) {
                lvlb.wall(x - 1, z, 1.5 * PI);
            }
        }
    }

    let (spawn_x, spawn_z) = spawnpoint.expect("Map is missing the spawnpoint");

    vec3(spawn_x, 0.0, spawn_z)
}

fn parse_map(map: &str) -> HashMap<(usize, usize), Tile> {
    let mut tiles = HashMap::new();

    for (z, line) in map.lines().skip(1).enumerate() {
        for (x, tile) in line.chars().enumerate() {
            if let Some(tile) = Tile::new(tile) {
                tiles.insert((x, z), tile);
            }
        }
    }

    tiles
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Floor,
    Wall,
    Light,
    Spawnpoint,
    Sphere(SphereTile),
}

impl Tile {
    fn new(tile: char) -> Option<Self> {
        match tile {
            '.' => Some(Self::Floor),
            '#' => Some(Self::Wall),
            '\'' => Some(Self::Light),
            '*' => Some(Self::Spawnpoint),
            'R' => Some(Self::Sphere(SphereTile::Red)),
            'G' => Some(Self::Sphere(SphereTile::Green)),
            'B' => Some(Self::Sphere(SphereTile::Blue)),
            ' ' => None,

            tile => {
                panic!("Unknown tile: `{tile}`");
            }
        }
    }

    fn is_wall(&self) -> bool {
        matches!(self, Self::Wall)
    }

    fn is_floorlike(&self) -> bool {
        matches!(
            self,
            Self::Floor | Self::Light | Self::Spawnpoint | Self::Sphere(_)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SphereTile {
    Red,
    Green,
    Blue,
}

impl SphereTile {
    fn color(&self) -> Color {
        match self {
            SphereTile::Red => Color::rgb(1.0, 0.139, 0.139),
            SphereTile::Green => Color::rgb(0.139, 1.0, 0.139),
            SphereTile::Blue => Color::rgb(0.139, 0.139, 1.0),
        }
    }
}
