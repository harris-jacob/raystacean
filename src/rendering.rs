use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SceneMaterial>::default())
            .add_systems(Startup, setup);
    }
}

pub fn setup(
    mut materials: ResMut<Assets<SceneMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Single<&Window>,
) {
    let material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        camera_transform: Mat4::default(),
    });

    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material_handle)));

    // TODO: should this be here?
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        GlobalTransform::default(),
    ));
}

#[repr(C)]
#[derive(Clone, ShaderType)]
pub struct GpuBox {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 3],
    _padding: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SceneMaterial {
    #[uniform(0)]
    aspect_ratio: Vec2,
    #[uniform(1)]
    pub camera_transform: Mat4,
    // #[storage(2, read_only)]
    // pub boxes: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
pub struct SceneMaterialHandle(Handle<SceneMaterial>);

impl SceneMaterialHandle {
    pub fn get_mut<'a>(&self, assets: &'a mut Assets<SceneMaterial>) -> &'a mut SceneMaterial {
        assets.get_mut(&self.0).expect("SceneMaterial should exist")
    }
}

impl Material for SceneMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}
