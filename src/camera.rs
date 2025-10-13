use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::view::RenderLayers;

use crate::transform_ext::CameraViewMatrix;
use crate::{controls, layers, rendering};

pub struct CameraPlugin;

#[derive(Debug, Component)]
pub struct MainCamera;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraControls::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    orbit_camera_input,
                    zoom_camera_input,
                    pan_camera_input,
                    update_material_transform,
                    update_main_camera,
                ),
            );
    }
}

#[derive(Resource, Debug)]
struct CameraControls {
    target: Vec3,
    azimuth: f32,
    distance: f32,
    elevation: f32,
}

impl Default for CameraControls {
    fn default() -> Self {
        Self {
            target: Vec3::new(0.0, 0.0, 0.0),
            azimuth: 20.0,
            distance: 10.0,
            elevation: std::f32::consts::FRAC_PI_4,
        }
    }
}

impl CameraControls {
    pub fn transform(&self) -> Transform {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.azimuth, self.elevation, 0.0);
        let camera_offset = rotation * Vec3::new(0.0, 0.0, -self.distance);
        let camera_position = self.target + camera_offset;

        Transform::from_translation(camera_position).looking_at(self.target, Vec3::Y)
    }
}

fn setup(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(layers::GIZMOS_LAYER);

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Camera {
            order: layers::GIZMOS_CAMERA,
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_2,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }),
        RenderLayers::layer(layers::GIZMOS_LAYER),
    ));
}

fn orbit_camera_input(
    control_intent: Res<controls::ControlIntent>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera: ResMut<CameraControls>,
) {
    if *control_intent != controls::ControlIntent::Orbitting {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    let sensitivity = 0.005;
    camera.azimuth -= delta.x * sensitivity;
    camera.elevation = (camera.elevation + delta.y * sensitivity).clamp(
        std::f32::consts::FRAC_PI_8,
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
    control_intent: Res<controls::ControlIntent>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera: ResMut<CameraControls>,
) {
    if *control_intent != controls::ControlIntent::Panning {
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

    camera.target += pan_right + pan_forward;
}

fn update_main_camera(
    controls: ResMut<CameraControls>,
    mut query: Query<(&MainCamera, &mut Transform)>,
) {
    let (_, mut camera_transform) = query.single_mut().expect("should be one main camera");

    *camera_transform = controls.transform();
}

fn update_material_transform(
    query: Query<(&Transform, &Projection), With<MainCamera>>,
    mut lit_material: ResMut<Assets<rendering::LitMaterial>>,
    mut selection_material: ResMut<Assets<rendering::SelectionMaterial>>,
) {
    let (camera_transform, projection) = query.single().expect("should be one main camera");

    for (_, material) in lit_material.iter_mut() {
        material.view_to_world = camera_transform.view_matrix().inverse();
        material.clip_to_view = projection.get_clip_from_view().inverse();
    }
    for (_, material) in selection_material.iter_mut() {
        material.view_to_world = camera_transform.view_matrix().inverse();
        material.clip_to_view = projection.get_clip_from_view().inverse();
    }
}
