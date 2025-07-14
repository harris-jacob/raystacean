use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

pub fn setup(
    mut materials: ResMut<Assets<SceneMaterial>>,
    mut commands: Commands,
    window: Single<&Window>,
) {
    let material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        camera_transform: Mat4::default(),
    });

    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));
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
