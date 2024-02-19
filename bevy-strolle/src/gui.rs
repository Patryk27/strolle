use bevy::core_pipeline::fxaa;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
pub struct SimpleGuiPlugin;

impl Plugin for SimpleGuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuiConfig>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, gui)
            .add_systems(Update, switch_mode);
    }
}
#[derive(Resource, Debug, Default)]
struct GuiConfig {
    // example
    fxaa: bool,
    fxaa_changed: bool,
    tonemapping: Tonemapping,
    tonemapping_changed: bool,
}
fn gui(
    mut contexts: EguiContexts,
    render_config: ResMut<GuiConfig>,
    point_lights: Query<&mut PointLight>,
    materials: ResMut<Assets<StandardMaterial>>,
    query_material_handles: Query<&mut Handle<StandardMaterial>>,
    time: Res<Time>,
) {
    egui::Window::new("Gui")
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            pointlights_ui(ui, point_lights);
            materials_ui(ui, materials, query_material_handles);
            ui.separator();

            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    faxx_tonemapping_ui(ui, render_config);
                });

            ui.separator();

            fps_ui(ui, time);
        });
}

fn pointlights_ui(ui: &mut egui::Ui, mut point_lights: Query<&mut PointLight>) {
    ui.collapsing("PointLights", |ui| {
        let mut num: i32 = 0;
        for mut light in point_lights.iter_mut() {
            ui.collapsing(format!("PointLight {}", num), |ui| {
                ui.horizontal(|ui| {
                    ui.label("intensity");
                    ui.add(egui::Slider::new(
                        &mut light.intensity,
                        0.0..=10000.0,
                    ));
                });
                ui.collapsing(format!("full info"), |ui| {
                    ui.label(format!("{:?}", light));
                });
            });
            num += 1;
        }
    });
}

fn materials_ui(
    ui: &mut egui::Ui,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query_material_handles: Query<&mut Handle<StandardMaterial>>,
) {
    ui.collapsing("Materials", |ui| {
        let mut num = 0;
        for material_handle in query_material_handles.iter() {
            ui.collapsing(format!("matrial {}", num), |ui| {
                if let Some(material) = materials.get_mut(material_handle) {
                    let rgba = material.base_color.as_rgba_f32();
                    let mut rgb = [rgba[0],rgba[1],rgba[2]];
                    ui.color_edit_button_rgb(&mut rgb);
                    material.base_color = Color::rgba(rgb[0],rgb[1],rgb[2],rgba[3]);
                    ui.collapsing(format!("full info"), |ui| {
                        if let Some(material) = materials.get(material_handle) {
                            ui.label(format!("{:?}", material));
                        }
                    });
                }
            });
            num += 1;
        }
    });
}

fn faxx_tonemapping_ui(
    ui: &mut egui::Ui,
    mut render_config: ResMut<GuiConfig>,
) {
    ui.label("FAXX");
    render_config.fxaa_changed |=
        ui.checkbox(&mut render_config.fxaa, "").changed();
    ui.end_row();
    ui.label("ToneMapping");
    let before = render_config.tonemapping;
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
    if before != render_config.tonemapping {
        render_config.tonemapping_changed = true;
    }
}
fn fps_ui(ui: &mut egui::Ui, time: Res<Time>) {
    let fps_text = format!(
        "{} FPS ({:.2} ms/frame) ",
        (1.0 / time.delta_seconds()).floor(),
        1000.0 * time.delta_seconds()
    );
    ui.label(fps_text);
}

fn switch_mode(
    mut commands: Commands,
    mut render_config: ResMut<GuiConfig>,
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
