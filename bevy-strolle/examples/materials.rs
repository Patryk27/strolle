use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::{
    AddressMode, Extent3d, SamplerDescriptor, TextureDimension, TextureFormat,
};
use bevy::render::texture::ImageSampler;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 768.0,
                height: 512.0,
                mode: WindowMode::Windowed,
                ..default()
            },
            ..default()
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(StrollePlugin)
        .add_startup_system(setup)
        .add_system(process_input)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut strolle_materials: ResMut<Assets<StrolleMaterial>>,
) {
    let striped_image = images.add(striped_image());
    let mut mesh = Mesh::from(shape::Plane { size: 1.0 });

    let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) else {
        unreachable!();
    };

    for uv in uvs {
        uv[1] *= 50.0;
    }

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(striped_image),
            ..default()
        }),
        transform: Transform::from_xyz(2.0, 1.0, 0.0)
            .with_rotation(
                Quat::from_rotation_z(-PI / 2.0) * Quat::from_rotation_x(PI),
            )
            .with_scale(vec3(2.0, 1.0, 10.0)),
        ..default()
    });

    // ---

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_scale(vec3(4.0, 1.0, 10.0)),
        ..default()
    });

    // ---

    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 1.0,
        subdivisions: 6,
    }));

    let mut spheres = Vec::new();

    for sphere_id in 0..7 {
        let sphere = commands.spawn(MaterialMeshBundle {
            mesh: sphere_mesh.clone(),
            material: strolle_materials.add(build_material(0, sphere_id)),
            transform: Transform::from_xyz(
                0.0,
                0.5,
                ((sphere_id as f32) - 3.0) * 1.1,
            )
            .with_scale(Vec3::splat(0.5)),
            ..default()
        });

        spheres.push(sphere.id());
    }

    // ---

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 4.0, 0.0),
        ..default()
    });

    // ---

    commands
        .spawn(Camera3dBundle {
            camera_render_graph: CameraRenderGraph::new(
                bevy_strolle::graph::NAME,
            ),
            ..default()
        })
        .insert(StrolleCamera {
            config: st::ViewportConfiguration {
                bounces: 1,
                ..default()
            },
        })
        .insert(OrbitCameraBundle::new(
            {
                let mut controller = OrbitCameraController::default();

                controller.mouse_rotate_sensitivity = Vec2::ONE * 0.2;
                controller.mouse_translate_sensitivity = Vec2::ONE * 0.5;
                controller
            },
            vec3(-7.0, 1.5, 0.0),
            vec3(0.0, 0.5, 0.0),
        ));

    commands.insert_resource(State {
        spheres,
        material_id: 0,
    });
}

#[derive(Resource)]
struct State {
    spheres: Vec<Entity>,
    material_id: usize,
}

fn process_input(
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    mut strolle_material_handles: Query<&mut Handle<StrolleMaterial>>,
    mut strolle_materials: ResMut<Assets<StrolleMaterial>>,
) {
    const MAX_MATERIALS: usize = 3;

    let mut reload_materials = false;

    if keys.just_pressed(KeyCode::Left) {
        reload_materials = true;

        state.material_id = state
            .material_id
            .checked_sub(1)
            .unwrap_or(MAX_MATERIALS - 1);
    }

    if keys.just_pressed(KeyCode::Right) {
        reload_materials = true;
        state.material_id = (state.material_id + 1) % MAX_MATERIALS;
    }

    if reload_materials {
        for (sphere_id, &sphere_entity) in state.spheres.iter().enumerate() {
            let sphere_material = strolle_materials
                .add(build_material(state.material_id, sphere_id));

            *strolle_material_handles.get_mut(sphere_entity).unwrap() =
                sphere_material;
        }
    }
}

fn build_material(material_id: usize, sphere_id: usize) -> StrolleMaterial {
    let sphere_n = (sphere_id as f32) / 7.0;
    let mut base_color = Color::hsl(sphere_n * 360.0, 0.5, 0.5);

    match material_id {
        0 => {
            base_color.set_a(0.0);

            StrolleMaterial {
                parent: StandardMaterial {
                    base_color,
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: sphere_n / 2.0,
                    ..default()
                },
                refraction: 1.1,
                ..default()
            }
        }

        1 => {
            base_color.set_a(0.0);

            StrolleMaterial {
                parent: StandardMaterial {
                    base_color,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                },
                refraction: 1.0 + sphere_n / 10.0,
                ..default()
            }
        }

        2 => {
            base_color.set_a(0.90);

            StrolleMaterial {
                parent: StandardMaterial {
                    base_color,
                    alpha_mode: AlphaMode::Blend,
                    reflectance: 0.0,
                    ..default()
                },
                ..default()
            }
        }

        _ => unreachable!(),
    }
}

/// Returns an image with black and white stripes; used for the rectangle in the
/// background.
fn striped_image() -> Image {
    let mut img = Image::new_fill(
        Extent3d {
            width: 1,
            height: 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255, 255, 255, 255, 255],
        TextureFormat::Rgba8Unorm,
    );

    img.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        ..default()
    });

    img
}
