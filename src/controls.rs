use bevy::prelude::*;

pub struct ControlContextPlugin;

impl Plugin for ControlContextPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ControlMode::Select)
            .insert_resource(ControlIntent::None)
            .add_systems(Update, resolve_control_intent);
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Select,
    PlaceGeometry,
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ControlIntent {
    Panning,
    Orbitting,
    ContextAction,
    None,
}

/// Centralized system for resolving control intent, ensuring that only one user
/// action is performed at a time. Returns None if the user is not pressing an
/// actively captured input combination.
///
/// TODO: remove this system
pub fn resolve_control_intent(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut intent: ResMut<ControlIntent>,
) {
    let action = if is_orbitting(&mouse, &keys) {
        ControlIntent::Orbitting
    } else if is_panning(&mouse, &keys) {
        ControlIntent::Panning
    } else if mouse.pressed(MouseButton::Left) {
        ControlIntent::ContextAction
    } else {
        ControlIntent::None
    };

    *intent = action;
}

fn is_orbitting(mouse: &ButtonInput<MouseButton>, keys: &ButtonInput<KeyCode>) -> bool {
    let ctrl_down = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    (ctrl_down && mouse.pressed(MouseButton::Left)) || mouse.pressed(MouseButton::Right)
}

fn is_panning(mouse: &ButtonInput<MouseButton>, keys: &ButtonInput<KeyCode>) -> bool {
    let shift_down = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    (shift_down && mouse.pressed(MouseButton::Right)) || mouse.pressed(MouseButton::Middle)
}
