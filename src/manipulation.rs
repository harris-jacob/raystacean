use bevy::prelude::*;

use crate::{camera, events, geometry, selection};

pub struct ManipulationPlugin;

impl Plugin for ManipulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_drag_to_selection);
    }
}

fn apply_drag_to_selection(
    mut drag_events: EventReader<events::OriginDragged>,
    mut selected: Query<&mut geometry::BoxGeometry, With<selection::Selected>>,
    camera: Query<(&GlobalTransform, &Camera), With<camera::MainCamera>>,
) {
    for event in drag_events.read() {
        let mut geometry = selected.single_mut().expect("single");
        let (camera_transform, camera) = camera.single().expect("single");
        if let Some(delta_scalar) = axis_drag_scalar(
            camera,
            camera_transform,
            geometry.position,
            event.axis,
            event.delta,
        ) {
            geometry.position += event.axis.normalize() * delta_scalar * 0.05;
        }
    }
}

/// Projects the drag axis into screen space and projects onto the mouse_delta
/// (dot product) to find a signed scalar which represents the magnitude of
/// a drag along an axis.
fn axis_drag_scalar(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    obj_pos: Vec3,
    axis: Vec3,
    mouse_delta: Vec2,
) -> Option<f32> {
    let screen_obj = camera.world_to_viewport(cam_transform, obj_pos).ok()?;
    let screen_axis_pt = camera
        .world_to_viewport(cam_transform, obj_pos + axis)
        .ok()?;

    let axis_2d = (screen_axis_pt - screen_obj).trunc();
    
    if axis_2d.length_squared() < 1e-6 {
        return None;
    }

    let axis_2d_norm = axis_2d.normalize();

    // scalar = projection of mouse delta onto axis direction
    let scalar = mouse_delta.dot(axis_2d_norm);

    Some(scalar)
}
