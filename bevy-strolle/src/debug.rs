use bevy::core_pipeline::fxaa;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::utils::color_to_vec4;

pub struct StrolleDebugPlugin;

impl Plugin for StrolleDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, draw)
            .add_systems(Update, apply_changes);
    }
}

#[derive(Resource, Debug, Default)]
struct DebugState {
    fxaa: bool,
    fxaa_changed: bool,
    tonemapping: Tonemapping,
    tonemapping_changed: bool,
}

fn draw(
    mut contexts: EguiContexts,
    render_config: ResMut<DebugState>,
    point_lights: Query<&mut PointLight>,
    materials: ResMut<Assets<StandardMaterial>>,
    query_material_handles: Query<&mut Handle<StandardMaterial>>,
    time: Res<Time>,
) {
    egui::Window::new("Debug")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            draw_lights(ui, point_lights);
            draw_materials(ui, materials, query_material_handles);

            ui.separator();

            egui::Grid::new("grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    faxx_tonemapping_ui(ui, render_config);
                });

            ui.separator();

            ui.label(format!(
                "{} FPS ({:.2} ms/frame) ",
                (1.0 / time.delta_seconds()).floor(),
                1000.0 * time.delta_seconds()
            ));
        });
}

fn draw_lights(ui: &mut egui::Ui, mut point_lights: Query<&mut PointLight>) {
    ui.collapsing("Lights", |ui| {
        for (light_idx, mut light) in point_lights.iter_mut().enumerate() {
            ui.collapsing(format!("Light {}", light_idx), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Intensity");

                    ui.add(egui::Slider::new(
                        &mut light.intensity,
                        0.0..=10000.0,
                    ));
                });

                ui.collapsing("Info", |ui| {
                    ui.label(format!("{:#?}", light));
                });
            });
        }
    });
}

fn draw_materials(
    ui: &mut egui::Ui,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_handles: Query<&mut Handle<StandardMaterial>>,
) {
    ui.collapsing("Materials", |ui| {
        for (mat_idx, mat_handle) in material_handles.iter().enumerate() {
            ui.collapsing(format!("Material {}", mat_idx), |ui| {
                let Some(material) = materials.get_mut(mat_handle) else {
                    return;
                };

                let rgba = color_to_vec4(material.base_color);
                let mut rgb = [rgba[0], rgba[1], rgba[2]];

                ui.color_edit_button_rgb(&mut rgb);

                material.base_color =
                    Color::rgba(rgb[0], rgb[1], rgb[2], rgba[3]);

                ui.collapsing("Info", |ui| {
                    ui.label(format!("{:#?}", material));
                });
            });
        }
    });
}

fn faxx_tonemapping_ui(
    ui: &mut egui::Ui,
    mut render_config: ResMut<DebugState>,
) {
    ui.label("FXAA");

    render_config.fxaa_changed |=
        ui.checkbox(&mut render_config.fxaa, "").changed();

    ui.end_row();
    ui.label("Tonemapping");

    let curr_value = render_config.tonemapping;

    egui::ComboBox::from_label("")
        .selected_text(format!("{:?}", render_config.tonemapping))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::None,
                "None",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::AcesFitted,
                "AcesFitted",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::AgX,
                "AgX",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::BlenderFilmic,
                "BlenderFilmic",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::Reinhard,
                "Reinhard",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::ReinhardLuminance,
                "ReinhardLuminance",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::SomewhatBoringDisplayTransform,
                "SomewhatBoringDisplayTransform",
            );
            ui.selectable_value(
                &mut render_config.tonemapping,
                Tonemapping::TonyMcMapface,
                "TonyMcMapface",
            );
        });

    ui.end_row();

    if render_config.tonemapping != curr_value {
        render_config.tonemapping_changed = true;
    }
}

fn apply_changes(
    mut commands: Commands,
    mut render_config: ResMut<DebugState>,
    cameras: Query<Entity, With<Camera>>,
) {
    if render_config.fxaa_changed {
        for camera in &cameras {
            if render_config.fxaa {
                commands.entity(camera).insert(fxaa::Fxaa::default());
            } else {
                commands.entity(camera).remove::<fxaa::Fxaa>();
            }
        }

        render_config.fxaa_changed = !render_config.fxaa_changed;
    }

    if render_config.tonemapping_changed {
        for camera in &cameras {
            commands.entity(camera).insert(render_config.tonemapping);
        }

        render_config.tonemapping_changed = !render_config.tonemapping_changed;
    }
}
