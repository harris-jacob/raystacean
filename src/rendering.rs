use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BufferUsages, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat, TextureUsages
};
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::view::RenderLayers;

use crate::{events, selection};
use crate::{geometry, layers};

pub struct RenderingPlugin;

#[derive(Resource)]
pub struct ShaderBufferHandle(Handle<ShaderStorageBuffer>);

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SceneMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (boxes_to_gpu, cursor_position));
    }
}

// TODO: ordering is a mess
pub fn setup(
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<SceneMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Single<&Window>,
) {
    let size = Extent3d {
        width: window.width().round() as u32,
        height: window.height().round() as u32,
        ..default()
    };

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);
    let boxes = buffers.add(ShaderStorageBuffer::default());

    let selection_buffer = vec![0.0; 3];
    let mut selection_buffer = ShaderStorageBuffer::from(selection_buffer);
    selection_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        is_color_picking: bool_to_gpu(false),
        boxes: boxes.clone(),
        cursor_position: Vec2::default(),
        // Selection buffer is not needed here
        selection: buffers.add(selection_buffer),
    });

    commands.insert_resource(ShaderBufferHandle(boxes));
    commands.insert_resource(SceneMaterialHandle(material_handle.clone()));

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    commands
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            RenderLayers::layer(layers::SHADER_LAYER),
        ))
        .observe(output_click_event);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: layers::TEXTURE_CAMERA,
            target: image_handle.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        RenderLayers::layer(layers::SHADER_LAYER),
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: layers::SHADER_CAMERA,
            ..default()
        },
        RenderLayers::layer(layers::SHADER_LAYER),
    ));

    commands.spawn((
        Sprite::from_image(image_handle),
        RenderLayers::layer(layers::SHADER_LAYER),
    ));
}

fn output_click_event(trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    // TODO: does this belong here?
    if trigger.button != PointerButton::Primary {
        return;
    }

    commands.trigger(events::PlaneClicked);
}

fn boxes_to_gpu(
    boxes: Query<(&geometry::BoxGeometry, Has<selection::Selected>)>,
    buffer_handle: Res<ShaderBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let buffer = buffer_handle.get_mut(&mut buffers);

    let gpu_data: Vec<GpuBox> = boxes
        .iter()
        .map(|(b, selected)| GpuBox {
            position: b.position.into(),
            scale: b.scale.into(),
            color: b.id.to_color(),
            logical_color: b.id.to_color(),
            selected: bool_to_gpu(selected),
            ..default()
        })
        .collect();

    buffer.set_data(gpu_data);
}

pub fn bool_to_gpu(value: bool) -> u32 {
    if value { 1 } else { 0 }
}

fn cursor_position(
    windows: Query<&Window>,
    scene_material: Res<SceneMaterialHandle>,
    mut materials: ResMut<Assets<SceneMaterial>>,
) {
    let window = windows.single().expect("single");
    let scene_material = scene_material.get_mut(&mut materials);

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Cursor position to ndc
    scene_material.cursor_position = Vec2::new(
        cursor_pos.x / window.width() * 2.0 - 1.0,
        (cursor_pos.y / window.height() * 2.0) - 1.0,
    );
}

#[repr(C)]
#[derive(Clone, ShaderType, Default)]
pub struct GpuBox {
    pub position: [f32; 3],
    _pad1: f32,
    pub scale: [f32; 3],
    _pad2: f32,
    pub color: [f32; 3],
    _pad3: f32,
    pub logical_color: [f32; 3],
    pub selected: u32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SceneMaterial {
    #[uniform(0)]
    pub aspect_ratio: Vec2,
    #[uniform(1)]
    pub view_to_world: Mat4,
    #[uniform(2)]
    pub clip_to_view: Mat4,
    #[uniform(3)]
    pub cursor_position: Vec2,
    #[uniform(4)]
    pub is_color_picking: u32,
    #[storage(5, read_only)]
    pub boxes: Handle<ShaderStorageBuffer>,
    #[storage(6)]
    pub selection: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
pub struct SceneMaterialHandle(Handle<SceneMaterial>);

impl ShaderBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }

    pub fn inner(&self) -> &Handle<ShaderStorageBuffer> {
        &self.0
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
