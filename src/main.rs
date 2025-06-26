use std::f32::consts::PI;

use bevy::input::mouse::MouseMotion;
use bevy::log::LogPlugin;
use bevy::math::prelude::Plane3d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    aspect_ratio: Vec2,
    #[uniform(1)]
    camera_rotation: Mat3,
}

impl Material for CustomMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/custom_material.wgsl".into()
    // }
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                ..default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera_input, update_material_rotation))
        .run();
}

#[derive(Component, Debug)]
struct OrbitCamera {
    azimuth: f32,
    elevation: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    window: Single<&Window>,
) {
    let material_handle = materials.add(CustomMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        camera_rotation: Mat3::from_rotation_x(PI / 10.0),
    });

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    // Background
    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material_handle)));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        GlobalTransform::default(),
        OrbitCamera {
            // target: Vec3::ZERO,
            azimuth: 0.0,
            elevation: 0.0,
        },
    ));
}

fn update_material_rotation(
    query: Query<&OrbitCamera>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let orbit = query.single().expect("Material rotation");

    for mat in materials.iter_mut() {
        mat.1.camera_rotation = rotation_from_azimuth_elevation(orbit.azimuth, orbit.elevation);
    }
}

fn orbit_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<&mut OrbitCamera>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    for mut orbit in query.iter_mut() {
        let sensitivity = 0.005;
        orbit.azimuth -= delta.x * sensitivity;
        orbit.elevation = (orbit.elevation + delta.y * sensitivity).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }
}

fn rotation_from_azimuth_elevation(azimuth: f32, elevation: f32) -> Mat3 {
    let yaw = Quat::from_rotation_y(azimuth);
    let pitch = Quat::from_rotation_x(elevation);
    let rotation = pitch * yaw;
    Mat3::from_quat(rotation).inverse()
}
