use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::log::LogPlugin;
use bevy::math::prelude::Plane3d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::render::storage::ShaderStorageBuffer;

#[repr(C)]
#[derive(Clone, Copy, ShaderType)]
pub struct GpuBox {
    pub position: [f32; 3],
    pub size: f32, // uniform scale
    pub color: [f32; 3],
    _padding: f32,
}

impl Default for GpuBox {
    fn default() -> Self {
        GpuBox {
            position: [0.0, 0.0, -2.0],
            size: 1.0,
            color: [255.0, 0.0, 0.0],
            _padding: 0.0,
        }
    }
}

impl GpuBox {
    fn with_position(self, pos: Vec3) -> Self {
        GpuBox {
            position: [pos.x, pos.y, pos.z],
            ..self
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    aspect_ratio: Vec2,
    #[uniform(1)]
    camera_transform: Mat4,
    #[storage(2, read_only)]
    pub boxes: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
struct CustomMaterialHandle(Handle<CustomMaterial>);

#[derive(Resource, Deref)]
struct BoxStorageHandle(Handle<ShaderStorageBuffer>);

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
                place_box_system,
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

impl OrbitControls {
    pub fn transform(&self) -> Mat4 {
        let rotation = Quat::from_euler(EulerRot::YXZ, self.azimuth, self.elevation, 0.0);

        let camera_offset = rotation * Vec3::new(0.0, 0.0, self.distance);
        let camera_position = self.target + camera_offset;

        Mat4::look_at_lh(camera_position, self.target, Vec3::Y)
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    window: Single<&Window>,
) {
    let boxes = vec![GpuBox::default()];

    let boxes = buffers.add(ShaderStorageBuffer::from(boxes));

    commands.insert_resource(BoxStorageHandle(boxes.clone()));

    let material_handle = materials.add(CustomMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        camera_transform: Mat4::default(),
        boxes,
    });

    commands.insert_resource(CustomMaterialHandle(material_handle.clone()));

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
        mat.1.camera_transform = orbit_controls.transform().inverse();
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

        controls.target += pan_right - pan_forward;
    }
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

fn place_box_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    orbit_controls: Query<&OrbitControls>,
    box_handle: Res<BoxStorageHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = windows.single().expect("single");
    let orbit_controls = orbit_controls.single().expect("single");
    let buffer = buffers.get_mut(&box_handle.0).expect("exists");

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // TODO: tidy this math up and write some comments
    let screen_size = window.size();
    let ndc = (cursor_pos / screen_size) * 2.0 - Vec2::ONE;
    let pixel_coords = ndc * Vec2::new(window.width() / window.height(), 1.0);

    let ray_dir_camera_space = Vec3::new(pixel_coords.x, pixel_coords.y, 1.0).normalize();

    let camera_inv = orbit_controls.transform().inverse();

    let ray_dir = (camera_inv * ray_dir_camera_space.extend(0.0))
        .truncate()
        .normalize();
    let ray_origin = (camera_inv * Vec4::new(0.0, 0.0, 0.0, 1.0)).truncate();

    // Intersect with ground plane (Y=0)
    let t = -ray_origin.y / ray_dir.y;

    if t < 0.0 {
        return;
    }

    let hit = ray_origin + ray_dir * t;

    let new_box = GpuBox::default().with_position(hit);

    buffer.set_data(vec![new_box]);
}
