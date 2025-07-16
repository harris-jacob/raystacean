use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use crate::rendering;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraControls::default()).add_systems(
            Update,
            (
                orbit_camera_input,
                zoom_camera_input,
                pan_camera_input,
                update_material_transform,
            ),
        );
    }
}

#[derive(Resource, Debug)]
pub struct CameraControls {
    target: Vec3,
    azimuth: f32,
    distance: f32,
    elevation: f32,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 0.0, 0.0),
            azimuth: 0.0,
            distance: 10.0,
            elevation: std::f32::consts::FRAC_PI_4,
        }
    }
}

impl CameraControls {
    pub fn transform(&self) -> Mat4 {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.azimuth, self.elevation, 0.0);

        let camera_offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        let camera_position = self.target + camera_offset;

        Mat4::look_at_lh(camera_position, self.target, Vec3::Y)
    }
}

fn orbit_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera: ResMut<CameraControls>,
) {
    if !is_orbit_button_pressed(&buttons, &keys) {
        return;
    }

    let mut delta = Vec2::ZERO;
    dbg!(delta);
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    let sensitivity = 0.005;
    camera.azimuth -= delta.x * sensitivity;
    camera.elevation = (camera.elevation + delta.y * sensitivity).clamp(
        -std::f32::consts::FRAC_PI_2 + 0.01,
        std::f32::consts::FRAC_PI_2 - 0.01,
    );
}

fn zoom_camera_input(mut motion_evr: EventReader<MouseWheel>, mut camera: ResMut<CameraControls>) {
    let mut delta = 0.0;
    for ev in motion_evr.read() {
        delta += ev.y;
    }

    let sensitivity = 0.05;
    camera.distance = (camera.distance + delta * sensitivity).clamp(0.1, 20.0);
}

fn pan_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera: ResMut<CameraControls>,
) {
    if !is_pan_button_pressed(&buttons, &keys) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    let sensitivity = 0.05;

    let yaw = Quat::from_rotation_y(camera.azimuth);
    let pan_right = yaw * Vec3::X * delta.x * sensitivity;
    let pan_forward = yaw * Vec3::Z * delta.y * sensitivity;

    camera.target += pan_right - pan_forward;
}

fn update_material_transform(
    camera: ResMut<CameraControls>,
    material_handle: ResMut<rendering::SceneMaterialHandle>,
    mut materials: ResMut<Assets<rendering::SceneMaterial>>,
) {
    let material = material_handle.get_mut(&mut materials);

    material.camera_transform = camera.transform().inverse();
}

fn is_pan_button_pressed(buttons: &ButtonInput<MouseButton>, _keys: &ButtonInput<KeyCode>) -> bool {
    buttons.pressed(MouseButton::Right)
}

fn is_orbit_button_pressed(
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    let alt_down = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    (alt_down && buttons.pressed(MouseButton::Left)) || buttons.pressed(MouseButton::Middle)
}
