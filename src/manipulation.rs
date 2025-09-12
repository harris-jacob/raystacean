use bevy::prelude::*;

use crate::{events, geometry, selection};

pub struct ManipulationPlugin;

impl Plugin for ManipulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_drag_to_selection);
    }
}

fn apply_drag_to_selection(
    mut drag_events: EventReader<events::OriginDragged>,
    mut selected: Query<&mut geometry::BoxGeometry, With<selection::Selected>>,
) {
    for event in drag_events.read() {
        if let Ok(mut geometry) = selected.single_mut() {
            geometry.position += event.axis * event.delta * 0.1;
        }
    }
}
