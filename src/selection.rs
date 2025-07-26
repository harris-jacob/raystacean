use bevy::prelude::*;

use crate::{controls, events, geometry};

pub struct SelectionPlugin;

#[derive(Component)]
pub struct Selected;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, box_selection);
    }
}

fn box_selection(
    control_mode: Res<controls::ControlMode>,
    control_intent: Res<controls::ControlIntent>,
    selected: Query<Entity, With<Selected>>,
    boxes: Query<(Entity, &geometry::BoxGeometry)>,
    mut ev: EventReader<events::PixelColorUnderCursor>,
    mut commands: Commands,
) {
    if *control_mode != controls::ControlMode::Select
        || *control_intent != controls::ControlIntent::ContextAction
    {
        return;
    }

    // deselect existing
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    if let Some(latest) = ev.read().last() {
        let id = geometry::GeometryId::from_color(latest.color());

        if let Some(newly_selected) = boxes.iter().find(|(_, geometry)| geometry.id == id) {
            commands.entity(newly_selected.0).insert(Selected);
        }
    }
}
