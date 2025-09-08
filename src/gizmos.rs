use crate::{geometry, selection};
use bevy::prelude::*;

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_coordinate_system);
    }
}

// Draw a coordinate system for every box
fn draw_coordinate_system(
    boxes: Query<&geometry::BoxGeometry, With<selection::Selected>>,
    mut gizmos: Gizmos,
) {
    for b in boxes {
        gizmos.axes(Transform::from_translation(b.position), 2.0);
    }
}
