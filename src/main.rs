use bevy::math::prelude::Plane3d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Vec4,
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
        .add_plugins((DefaultPlugins, MaterialPlugin::<CustomMaterial>::default()))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    window: Single<&Window>,
) {
    let material_handle = materials.add(CustomMaterial {
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    });

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.height() * 0.5, window.width() * 0.5),
    )));

    // Background
    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material_handle)));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Z),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        GlobalTransform::default(),
    ));
}
