//! This file is not an example - it just contains common code used by some of
//! the examples.

#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_strolle::graph::StrolleGraph;
use bevy_strolle::prelude::*;
use smooth_bevy_cameras::controllers::fps::FpsCameraController;
use zip::ZipArchive;

pub fn extract_assets() {
    extract_asset("cornell");
    extract_asset("demo");
}

fn extract_asset(name: &str) {
    let dir = env::var("CARGO_MANIFEST_DIR")
        .expect("Please use `cargo` to run the examples");

    let dir = Path::new(&dir).join("assets");

    if dir.join(name).with_extension("obj").exists() {
        return;
    }

    let archive = dir.join(name).with_extension("zip");

    let archive = File::open(archive)
        .unwrap_or_else(|err| panic!("couldn't open asset {}: {}", name, err));

    let mut archive =
        ZipArchive::new(BufReader::new(archive)).unwrap_or_else(|err| {
            panic!("couldn't open archive for asset: {}: {}", name, err)
        });

    archive.extract(&dir).unwrap_or_else(|err| {
        panic!("couldn't extract asset {}: {}", name, err)
    });
}

// -----------------------------------------------------------------------------

pub fn handle_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut camera: Query<(
        &Transform,
        &mut CameraRenderGraph,
        &mut StrolleCamera,
        &mut FpsCameraController,
    )>,
) {
    let (camera_xform, mut camera_rg, mut camera, mut fps_camera_controller) =
        camera.single_mut();

    if keys.just_pressed(KeyCode::Digit1) {
        camera_rg.set(StrolleGraph);

        camera.mode = match camera.mode {
            st::CameraMode::Image { denoise } => {
                st::CameraMode::Image { denoise: !denoise }
            }
            _ => st::CameraMode::Image { denoise: true },
        };
    }

    if keys.just_pressed(KeyCode::Digit2) {
        camera_rg.set(StrolleGraph);

        camera.mode = match camera.mode {
            st::CameraMode::DiDiffuse { denoise } => {
                st::CameraMode::DiDiffuse { denoise: !denoise }
            }
            _ => st::CameraMode::DiDiffuse { denoise: true },
        };
    }

    if keys.just_pressed(KeyCode::Digit3) {
        camera_rg.set(StrolleGraph);

        camera.mode = match camera.mode {
            st::CameraMode::DiSpecular { denoise } => {
                st::CameraMode::DiSpecular { denoise: !denoise }
            }
            _ => st::CameraMode::DiSpecular { denoise: true },
        };
    }

    if keys.just_pressed(KeyCode::Digit4) {
        camera_rg.set(StrolleGraph);

        camera.mode = match camera.mode {
            st::CameraMode::GiDiffuse { denoise } => {
                st::CameraMode::GiDiffuse { denoise: !denoise }
            }
            _ => st::CameraMode::GiDiffuse { denoise: true },
        };
    }

    if keys.just_pressed(KeyCode::Digit5) {
        camera_rg.set(StrolleGraph);

        camera.mode = match camera.mode {
            st::CameraMode::GiSpecular { denoise } => {
                st::CameraMode::GiSpecular { denoise: !denoise }
            }
            _ => st::CameraMode::GiSpecular { denoise: true },
        };
    }

    if keys.just_pressed(KeyCode::Digit8) {
        camera_rg.set(StrolleGraph);

        camera.mode = st::CameraMode::BvhHeatmap;
    }

    if keys.just_pressed(KeyCode::Digit9) {
        camera_rg.set(StrolleGraph);

        camera.mode = st::CameraMode::Reference { depth: 1 };
    }

    if keys.just_pressed(KeyCode::Digit0) {
        camera_rg.set(Core3d);
    }

    if keys.just_pressed(KeyCode::Semicolon) {
        fps_camera_controller.enabled = !fps_camera_controller.enabled;

        let mut window = window.single_mut();

        window.cursor_options.visible = !fps_camera_controller.enabled;

        window.cursor_options.grab_mode = if fps_camera_controller.enabled {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }

    if keys.just_pressed(KeyCode::KeyX) {
        println!("{:?}", camera_xform.translation);
    }
}

// -----------------------------------------------------------------------------

#[derive(Resource)]
pub struct Sun {
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

pub fn handle_sun(keys: Res<ButtonInput<KeyCode>>, mut sun: ResMut<Sun>) {
    if keys.just_pressed(KeyCode::KeyH) {
        sun.azimuth -= 0.05;
    }

    if keys.just_pressed(KeyCode::KeyJ) {
        sun.altitude -= 0.05;
    }

    if keys.just_pressed(KeyCode::KeyK) {
        sun.altitude += 0.05;
    }

    if keys.just_pressed(KeyCode::KeyL) {
        sun.azimuth += 0.05;
    }
}

pub fn animate_sun(
    time: Res<Time>,
    mut st_sun: ResMut<StrolleSun>,
    mut sun: ResMut<Sun>,
) {
    if sun.initialized {
        st_sun.azimuth =
            st_sun.azimuth + (sun.azimuth - st_sun.azimuth) * time.delta_secs();

        st_sun.altitude = st_sun.altitude
            + (sun.altitude - st_sun.altitude) * time.delta_secs();
    } else {
        sun.initialized = true;
        st_sun.azimuth = sun.azimuth;
        st_sun.altitude = sun.altitude;
    }
}
