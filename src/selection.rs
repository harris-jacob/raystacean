use bevy::prelude::*;

use crate::{controls, events, geometry, node_id};

pub struct SelectionPlugin;

#[derive(Component)]
pub struct Selected;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_observer(box_selection).add_observer(deselect);
    }
}

fn deselect(
    _trigger: Trigger<events::Deselect>,
    selected: Query<Entity, With<Selected>>,
    mut commands: Commands,
) {
    deselect_selected(selected, &mut commands);
}

fn box_selection(
    _trigger: Trigger<events::PlaneClicked>,
    control_mode: Res<controls::ControlMode>,
    selected: Query<Entity, With<Selected>>,
    boxes: Query<(Entity, &geometry::BoxGeometry)>,
    ev: EventReader<events::PixelColorUnderCursor>,
    mut commands: Commands,
) {
    match control_mode.selection_policy() {
        controls::SelectionPolicy::None => {}
        controls::SelectionPolicy::Single => {
            deselect_selected(selected, &mut commands);

            select_under_cursor(ev, commands, boxes);
        }
        controls::SelectionPolicy::Multi(size) => {
            if selected.iter().len() > size {
                return;
            }

            select_under_cursor(ev, commands, boxes);
        }
    }
}

fn select_under_cursor(
    mut ev: EventReader<events::PixelColorUnderCursor>,
    mut commands: Commands,
    boxes: Query<(Entity, &geometry::BoxGeometry)>,
) {
    if let Some(latest) = ev.read().last() {
        let id = node_id::NodeId::from_color(latest.color());

        if let Some(newly_selected) = boxes.iter().find(|(_, geometry)| geometry.id == id) {
            commands.entity(newly_selected.0).insert(Selected);
        }
    }
}

fn deselect_selected(selected: Query<Entity, With<Selected>>, commands: &mut Commands) {
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }
}
