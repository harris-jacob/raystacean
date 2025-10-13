use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, RichText},
};

use crate::{
    controls, geometry, node_id,
    operations::{self, OperationsForest},
    selection,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default()).add_systems(
            EguiPrimaryContextPass,
            (
                toolbar_ui,
                inspector_ui,
                csg_tooltip,
                place_geometry_tooltop,
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
    mut operations: ResMut<OperationsForest>,
    mut control_mode: ResMut<controls::ControlMode>,
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

                        show_color_for_primative(ui, &mut operations, &mut selected);

                        ui.label("Rounding");
                        ui.add(egui::Slider::new(&mut selected.rounding, 0.0..=1.0));
                        ui.end_row();

                        let union_text = RichText::new("union").size(14.0);
                        let begin_union_button = egui::Button::new(union_text);

                        let subtract_text = RichText::new("subtract").size(14.0);
                        let begin_subtract_button = egui::Button::new(subtract_text);

                        ui.label("Actions");
                        ui.horizontal(|ui| {
                            if ui
                                .add(begin_union_button)
                                .on_hover_text("begin union")
                                .clicked()
                            {
                                *control_mode = controls::ControlMode::UnionSelect;
                            }
                            if ui
                                .add(begin_subtract_button)
                                .on_hover_text("begin subtract")
                                .clicked()
                            {
                                *control_mode = controls::ControlMode::SubtractSelect;
                            }
                        });
                        ui.end_row();
                    });
                });

            ui.add_space(12.0);

            ui.label(egui::RichText::new("Operations").heading());

            ui.add_space(4.0);

            egui::Grid::new("Operations list")
                .striped(true)
                .show(ui, |ui| {
                    show_operations_for_selected(ui, operations, selected);
                })
        });
    }

    Ok(())
}

fn csg_tooltip(mut contexts: EguiContexts, control_mode: Res<controls::ControlMode>) -> Result {
    let operation_label = match *control_mode {
        controls::ControlMode::Select => return Ok(()),
        controls::ControlMode::PlaceGeometry => return Ok(()),
        controls::ControlMode::UnionSelect => "Union",
        controls::ControlMode::SubtractSelect => "Subtract",
    };

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Union Mode")
        .title_bar(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_TOP, [0.0, 10.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Select a second primative to create a {operation_label}",
            ));
            ui.label("Press esc to cancel");
        });

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

// When involved in a CSG operation a primative's color is overwritten by the
// operation. We should show the primative of the 'highest order' CSG operation
// the primative is involved in (which will always be a root node).
fn show_color_for_primative(
    ui: &mut egui::Ui,
    operations: &mut OperationsForest,
    target: &mut geometry::BoxGeometry,
) {
    let operation = operations.find_root_mut(&target.id).expect("exists");

    ui.horizontal(|ui| {
        ui.label("Picker");

        match operation {
            operations::Node::Geometry(_) => {
                ui.color_edit_button_rgb(&mut target.color);
            }
            operations::Node::Subtract(operation) | operations::Node::Union(operation) => {
                ui.label("Picker");
                ui.color_edit_button_rgb(&mut operation.color);
            }
        }
    });

    ui.end_row();
}

fn show_operations_for_selected(
    ui: &mut egui::Ui,
    mut operations: ResMut<OperationsForest>,
    selected: Mut<geometry::BoxGeometry>,
) {
    for root in operations.roots.iter_mut() {
        show_operations_for_primative(ui, root, selected.id);
    }
}

fn show_operations_for_primative(
    ui: &mut egui::Ui,
    node: &mut operations::Node,
    target: node_id::NodeId,
) -> bool {
    match node {
        operations::Node::Geometry(id) => *id == target,
        operations::Node::Union(union) => {
            if show_operations_for_primative(ui, &mut union.left, target)
                || show_operations_for_primative(ui, &mut union.right, target)
            {
                ui.label(format!("Union {}", union.id));
                ui.add(egui::Slider::new(&mut union.blend, 0.0..=1.0));
                ui.end_row();
                return true;
            }
            false
        }
        operations::Node::Subtract(subtract) => {
            if show_operations_for_primative(ui, &mut subtract.left, target)
                || show_operations_for_primative(ui, &mut subtract.right, target)
            {
                ui.label(format!("Subtract {}", subtract.id));
                ui.add(egui::Slider::new(&mut subtract.blend, 0.0..=1.0));
                ui.end_row();
                return true;
            }
            false
        }
    }
}
