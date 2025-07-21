use bevy::platform::hash::FixedHasher;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, BufferUsages, ShaderRef, ShaderType};
use bevy::render::storage::ShaderStorageBuffer;
use std::hash::BuildHasher;

use crate::geometry;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SceneMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, boxes_to_gpu);
    }
}

fn setup(
    mut materials: ResMut<Assets<SceneMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    window: Single<&Window>,
) {
    let boxes = buffers.add(ShaderStorageBuffer::default());
    let selection_buffer = vec![0.0; 3];
    let mut selection_buffer = ShaderStorageBuffer::from(selection_buffer);
    selection_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let selection = buffers.add(selection_buffer);

    let material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        camera_transform: Mat4::default(),
        boxes: boxes.clone(),
        selection: selection.clone(),
    });

    commands.insert_resource(ShaderBufferHandle(boxes));
    commands.insert_resource(SelectionBufferHandle(selection));
    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material_handle)));

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

fn boxes_to_gpu(
    boxes: Query<&geometry::BoxGeometry>,
    buffer_handle: Res<ShaderBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let buffer = buffer_handle.get_mut(&mut buffers);

    let gpu_data: Vec<GpuBox> = boxes
        .iter()
        .map(|b| GpuBox {
            position: b.position.into(),
            size: b.size,
            color: b.id.to_color(),
            _padding: 0.0,
        })
        .collect();

    buffer.set_data(gpu_data);
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
    #[storage(2, read_only)]
    pub boxes: Handle<ShaderStorageBuffer>,

    #[storage(3)]
    pub selection: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
pub struct SceneMaterialHandle(Handle<SceneMaterial>);

#[derive(Resource)]
pub struct ShaderBufferHandle(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct SelectionBufferHandle(Handle<ShaderStorageBuffer>);

impl ShaderBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }
}

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
