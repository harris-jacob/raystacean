use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, RichText},
};

use crate::{controls, geometry, selection};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_systems(EguiPrimaryContextPass, (add_geometry_tool, inspector_ui));
    }
}

pub fn add_geometry_tool(
    mut contexts: EguiContexts,
    mut control_mode: ResMut<controls::ControlMode>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Area::new("toolbar".into())
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -10.0])
        .show(ctx, |ui| {
            let text = RichText::new("+").size(24.0).strong();

            let button = egui::Button::new(text)
                .min_size(egui::vec2(50.0, 50.0))
                .corner_radius(20.0);

            ui.horizontal(|ui| {
                if ui.add(button).on_hover_text("add geometry").clicked() {
                    *control_mode = controls::ControlMode::PlaceGeometry;
                }
            });
        });

    Ok(())
}

fn inspector_ui(
    mut contexts: EguiContexts,
    mut selected: Query<&mut geometry::BoxGeometry, With<selection::Selected>>,
) -> Result {
    let context = contexts.ctx_mut()?;
    if let Ok(mut selected) = selected.single_mut() {
        egui::Window::new("Box").show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_gray(30))
                .corner_radius(5.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    egui::Grid::new("properties")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Position");
                            ui.add(egui::DragValue::new(&mut selected.position.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.position.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.position.z).speed(0.1));
                            ui.end_row();

                            ui.label("Scale");
                            ui.add(egui::DragValue::new(&mut selected.scale.x).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.scale.y).speed(0.1));
                            ui.add(egui::DragValue::new(&mut selected.scale.z).speed(0.1));
                            ui.end_row();
                        });

                    egui::Grid::new("color").show(ui, |ui| {
                        ui.label("Picker");
                        ui.color_edit_button_rgb(&mut selected.color);
                        ui.end_row();
                    })
                });
        });
    }

    Ok(())
}
