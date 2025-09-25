use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
    egui::{self, RichText},
};

pub struct EguiUiPlugin;

impl Plugin for EguiUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(Tool::Select)
            .add_systems(EguiPrimaryContextPass, add_geometry_tool);
    }
}

// Define your tool enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum Tool {
    Select,
    Box,
}

pub fn add_geometry_tool(mut contexts: EguiContexts, mut tool: ResMut<Tool>) -> Result {
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
                    *tool = match *tool {
                        Tool::Select => Tool::Box,
                        Tool::Box => Tool::Select,
                    };
                }
            });
        });

    Ok(())
}
