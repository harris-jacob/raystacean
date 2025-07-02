use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::log::LogPlugin;
use bevy::math::prelude::Plane3d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    aspect_ratio: Vec2,
    #[uniform(1)]
    camera_transform: Mat4,
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
        .add_systems(
            Update,
            (
                orbit_camera_input,
                zoom_camera_input,
                pan_camera_input,
                update_material_transform,
            ),
        )
        .run();
}

#[derive(Component, Debug, Default)]
struct OrbitControls {
    target: Vec3,
    azimuth: f32,
    distance: f32,
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
        camera_transform: Mat4::default(),
    });

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    // Background
    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material_handle)));

    // camera
    let mut orbit_controls = OrbitControls::default();
    orbit_controls.distance = 5.0;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        GlobalTransform::default(),
        orbit_controls,
    ));
}

fn update_material_transform(
    query: Query<&OrbitControls>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let orbit_controls = query.single().expect("Material rotation");

    for mat in materials.iter_mut() {
        let rotation = Quat::from_euler(
            EulerRot::YXZ,
            orbit_controls.azimuth,
            orbit_controls.elevation,
            0.0,
        );
        
        let camera_offset = rotation * Vec3::new(0.0, 0.0, orbit_controls.distance);
        let camera_position = orbit_controls.target + camera_offset;

        mat.1.camera_transform =
            Mat4::look_at_lh(camera_position, orbit_controls.target, Vec3::Y).inverse();
    }
}

fn orbit_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<&mut OrbitControls>,
) {
    if !is_orbit_button_pressed(&buttons, &keys) {
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

fn zoom_camera_input(
    mut motion_evr: EventReader<MouseWheel>,
    mut query: Query<&mut OrbitControls>,
) {
    let mut delta = 0.0;
    for ev in motion_evr.read() {
        delta += ev.y;
    }

    for mut controls in query.iter_mut() {
        let sensitivity = 0.05;
        controls.distance = (controls.distance + delta * sensitivity).clamp(0.1, 10.0);
    }
}

fn pan_camera_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut query: Query<&mut OrbitControls>,
) {
    if !is_pan_button_pressed(&buttons, &keys) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in motion_evr.read() {
        delta += ev.delta;
    }

    for mut controls in query.iter_mut() {
        let sensitivity = 0.05;

        let yaw = Quat::from_rotation_y(controls.azimuth);
        let pan_right = yaw * Vec3::X * delta.x * sensitivity;
        let pan_forward = yaw * Vec3::Z * delta.y * sensitivity;

        controls.target += (pan_right - pan_forward);
    }
}

fn is_pan_button_pressed(buttons: &ButtonInput<MouseButton>, keys: &ButtonInput<KeyCode>) -> bool {
    let alt_down = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    !alt_down && buttons.pressed(MouseButton::Left)
}

fn is_orbit_button_pressed(
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    let alt_down = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    (alt_down && buttons.pressed(MouseButton::Left)) || buttons.pressed(MouseButton::Middle)
}
