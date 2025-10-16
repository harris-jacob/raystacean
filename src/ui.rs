use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, RichText},
};

use crate::{controls, geometry, selection};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default()).add_systems(
            EguiPrimaryContextPass,
            (
                toolbar_ui,
                inspector_ui,
                place_geometry_tooltop,
                diagnostics_ui,
            ),
        );
    }
}

pub fn toolbar_ui(
    mut contexts: EguiContexts,
    mut control_mode: ResMut<controls::ControlMode>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Area::new("toolbar".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -10.0])
        .show(ctx, |ui| {
            let add_text = RichText::new("➕").size(24.0).strong();

            let add_geometry_button = egui::Button::new(add_text)
                .min_size(egui::vec2(50.0, 50.0))
                .corner_radius(20.0);

            ui.horizontal(|ui| {
                if ui
                    .add(add_geometry_button)
                    .on_hover_text("add geometry")
                    .clicked()
                {
                    *control_mode = controls::ControlMode::PlaceGeometry;
                }
            });
        });

    Ok(())
}

fn inspector_ui(
    mut contexts: EguiContexts,
    mut selected: Query<&mut geometry::BoxGeometry, With<selection::Selected>>,
    control_mode: ResMut<controls::ControlMode>,
) -> Result {
    // We only want to show this ui in select mode
    if *control_mode != controls::ControlMode::Select {
        return Ok(());
    }

    let context = contexts.ctx_mut()?;
    if let Ok(mut selected) = selected.single_mut() {
        egui::Window::new("Box").show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_gray(30))
                .corner_radius(5.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    egui::Grid::new("properties").striped(true).show(ui, |ui| {
                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut selected.position.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.position.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.position.z).speed(0.1));
                        });
                        ui.end_row();

                        ui.label("Scale");
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut selected.scale.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.scale.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.scale.z).speed(0.1));
                        });
                        ui.end_row();

                        ui.label("Picker");
                        ui.color_edit_button_rgb(&mut selected.color);
                        ui.end_row();

                        ui.label("Rounding");
                        ui.add(egui::Slider::new(&mut selected.rounding, 0.0..=1.0));
                        ui.end_row();

                        ui.label("Blend");
                        ui.add(egui::Slider::new(&mut selected.blend, 0.0..=1.0));
                        ui.end_row();

                        ui.add(egui::Checkbox::new(&mut selected.is_subtract, "Subtract"));
                        ui.end_row();
                    });
                });
        });
    }

    Ok(())
}

fn place_geometry_tooltop(
    mut contexts: EguiContexts,
    control_mode: Res<controls::ControlMode>,
) -> Result {
    if *control_mode != controls::ControlMode::PlaceGeometry {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Add Geometry Mode")
        .title_bar(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
        .show(ctx, |ui| {
            ui.label("Select a spot on the plane to place geometry");
            ui.label("Press esc to cancel");
        });

    Ok(())
}

fn diagnostics_ui(mut contexts: EguiContexts, diagnostics: Res<DiagnosticsStore>) -> Result {
    let ctx = contexts.ctx_mut()?;

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed());

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed());

    egui::Window::new("Performance")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .resizable(false)
        .show(ctx, |ui| {
            if let Some(fps) = fps {
                ui.label(RichText::new(format!("FPS: {}", fps.round())).size(16.0));
            } else {
                ui.label("FPS: calculating…");
            }

            if let Some(ft) = frame_time {
                ui.label(format!("Frame time: {ft:.2} ms"));
            }
        });

    Ok(())
}
