use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_resource::{
    AsBindGroup, BufferUsages, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat,
    TextureUsages,
};
use bevy::render::storage::ShaderStorageBuffer;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

use crate::layers::SHADER_CAMERA;
use crate::{events, node_id, operations, selection};
use crate::{geometry, layers};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<SceneMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    boxes_to_gpu,
                    ops_to_gpu,
                    cursor_position,
                    window_resize_system,
                ),
            );
    }
}

#[derive(Component)]
struct RenderingPlane;

fn setup(
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

    let primatives = buffers.add(ShaderStorageBuffer::default());
    let operations = buffers.add(ShaderStorageBuffer::default());
    let op_roots = buffers.add(ShaderStorageBuffer::from(Vec::<u32>::new()));

    let selection_buffer = vec![0.0; 3];
    let mut selection_buffer = ShaderStorageBuffer::from(selection_buffer);
    selection_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let selection = buffers.add(selection_buffer);

    let lit_material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        is_color_picking: LIT_PASS,
        primatives: primatives.clone(),
        operations: operations.clone(),
        op_roots: op_roots.clone(),
        selection: selection.clone(),
        cursor_position: Vec2::default(),
    });

    let color_pick_material_handle = materials.add(SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        is_color_picking: COLOR_PICKING_PASS,
        primatives: primatives.clone(),
        operations: operations.clone(),
        op_roots: op_roots.clone(),
        selection: selection.clone(),
        cursor_position: Vec2::default(),
    });

    commands.spawn(Readback::buffer(selection)).observe(
        |trigger: Trigger<ReadbackComplete>, mut ev: EventWriter<events::PixelColorUnderCursor>| {
            let data: Vec<f32> = trigger.event().to_shader_type();

            ev.write(events::PixelColorUnderCursor::new(Vec3::new(
                data[0], data[1], data[2],
            )));
        },
    );

    commands.insert_resource(PrimativesBufferHandle(primatives));
    commands.insert_resource(OperationsBufferHandle(operations));
    commands.insert_resource(OpRootsBufferHandle(op_roots));

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    // Main rendering pass, render to the screen
    commands
        .spawn((
            RenderingPlane,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(lit_material_handle),
            RenderLayers::layer(layers::SHADER_LAYER),
        ))
        .observe(output_click_event);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: SHADER_CAMERA,
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

    // Selection pass, render to an image
    commands
        .spawn((
            RenderingPlane,
            Mesh3d(mesh),
            MeshMaterial3d(color_pick_material_handle),
            RenderLayers::layer(layers::SELECTION_LAYER),
        ))
        .observe(output_click_event);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: layers::SELECTION_CAMERA,
            target: image_handle.clone().into(),
            clear_color: Color::WHITE.into(),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
        RenderLayers::layer(layers::SELECTION_LAYER),
    ));
}

fn window_resize_system(
    mut resize_events: EventReader<WindowResized>,
    mut materials: ResMut<Assets<SceneMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Mesh3d, With<RenderingPlane>>,
) {
    for e in resize_events.read() {
        let width = e.width;
        let height = e.height;

        for (_, mat) in materials.iter_mut() {
            mat.aspect_ratio = Vec2::new(width, height);
        }

        for mesh in &mut query {
            if let Some(mesh) = meshes.get_mut(&mesh.0) {
                *mesh = Mesh::from(Plane3d::new(Vec3::Z, Vec2::new(width * 0.5, height * 0.5)));
            }
        }
    }
}

fn output_click_event(trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    if trigger.button != PointerButton::Primary {
        return;
    }

    commands.trigger(events::PlaneClicked);
}

fn ops_to_gpu(
    // TODO: trigger on event
    primatives: Query<&geometry::BoxGeometry>,
    forest: Res<operations::OperationsForest>,
    ops_buffer_handle: Res<OperationsBufferHandle>,
    roots_buffer_handle: Res<OpRootsBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let (gpu_ops, gpu_roots) = flatten_postorder(forest, &primatives);
    let ops_buffer = ops_buffer_handle.get_mut(&mut buffers);
    ops_buffer.set_data(gpu_ops);

    let roots_buffer = roots_buffer_handle.get_mut(&mut buffers);
    roots_buffer.set_data(gpu_roots);
}

fn boxes_to_gpu(
    boxes: Query<(&geometry::BoxGeometry, Has<selection::Selected>)>,
    buffer_handle: Res<PrimativesBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let buffer = buffer_handle.get_mut(&mut buffers);

    let gpu_data: Vec<GpuPrimative> = boxes
        .iter()
        .map(|(b, selected)| GpuPrimative {
            position: b.position.into(),
            scale: b.scale.into(),
            color: b.color,
            rounding_radius: b.rounding_radius(),
            logical_color: b.id.to_color(),
            selected: bool_to_gpu(selected),
            ..default()
        })
        .collect();

    buffer.set_data(gpu_data);
}

fn bool_to_gpu(value: bool) -> u32 {
    if value { 1 } else { 0 }
}

fn cursor_position(windows: Query<&Window>, mut materials: ResMut<Assets<SceneMaterial>>) {
    let window = windows.single().expect("single");

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    for (_, material) in materials.iter_mut() {
        // Cursor position to ndc
        material.cursor_position = Vec2::new(
            cursor_pos.x / window.width() * 2.0 - 1.0,
            (cursor_pos.y / window.height() * 2.0) - 1.0,
        );
    }
}

#[repr(C)]
#[derive(Clone, ShaderType, Default)]
pub struct GpuPrimative {
    pub position: [f32; 3],
    _pad1: f32,
    pub scale: [f32; 3],
    _pad2: f32,
    pub color: [f32; 3],
    pub rounding_radius: f32,
    pub logical_color: [f32; 3],
    pub selected: u32,
}

#[repr(C)]
#[derive(Clone, ShaderType, Default, Debug)]
struct GpuOp {
    kind: u32,            // 0 = primative, 1 = union, 2 = subtract
    left: u32,            // left child index into OP for union (invalid for primative)
    right: u32,           // left child index into OP for union (invalid for primative)
    primative_index: u32, // for primatives, index into primative buffer
    color: [f32; 3],
    blend: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SceneMaterial {
    #[uniform(0)]
    aspect_ratio: Vec2,
    #[uniform(1)]
    pub view_to_world: Mat4,
    #[uniform(2)]
    pub clip_to_view: Mat4,
    #[uniform(3)]
    pub cursor_position: Vec2,
    #[uniform(4)]
    pub is_color_picking: u32,
    #[storage(5, read_only)]
    pub primatives: Handle<ShaderStorageBuffer>,
    #[storage(6, read_only)]
    pub operations: Handle<ShaderStorageBuffer>,
    #[storage(7, read_only)]
    pub op_roots: Handle<ShaderStorageBuffer>,
    #[storage(8)]
    pub selection: Handle<ShaderStorageBuffer>,
}

const LIT_PASS: u32 = 0;
const COLOR_PICKING_PASS: u32 = 1;

#[derive(Resource)]
pub struct PrimativesBufferHandle(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct OperationsBufferHandle(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct OpRootsBufferHandle(Handle<ShaderStorageBuffer>);

impl PrimativesBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }
}

impl OperationsBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("OperationsStorageBuffer should exist")
    }
}

impl OpRootsBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("OpRootsStorageBuffer should exist")
    }
}

impl Material for SceneMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
}

fn flatten_postorder(
    forest: Res<operations::OperationsForest>,
    primatives: &Query<&geometry::BoxGeometry>,
) -> (Vec<GpuOp>, Vec<u32>) {
    let mut ops = Vec::new();
    let mut roots = Vec::new();

    for root in forest.roots.iter() {
        roots.push(flatten_node(root, primatives, &mut ops));
    }

    (ops, roots)
}

fn flatten_node(
    node: &operations::Node,
    primatives: &Query<&geometry::BoxGeometry>,
    out: &mut Vec<GpuOp>,
) -> u32 {
    match node {
        operations::Node::Geometry(node_id) => {
            let idx = geometry_index_by_id(primatives, node_id);

            let op_index = out.len() as u32;

            out.push(GpuOp {
                kind: 0,
                primative_index: idx,
                ..default()
            });

            op_index
        }
        operations::Node::Union(union) => {
            let l_idx = flatten_node(&union.left, primatives, out);
            let r_idx = flatten_node(&union.right, primatives, out);

            let op_index = out.len() as u32;

            out.push(GpuOp {
                kind: 1,
                left: l_idx,
                right: r_idx,
                blend: union.blend,
                color: union.color,
                ..default()
            });

            op_index
        }

        operations::Node::Subtract(subtract) => {
            let l_idx = flatten_node(&subtract.left, primatives, out);
            let r_idx = flatten_node(&subtract.right, primatives, out);

            let op_index = out.len() as u32;

            out.push(GpuOp {
                kind: 2,
                left: l_idx,
                right: r_idx,
                blend: subtract.blend,
                ..default()
            });

            op_index
        }
    }
}

fn geometry_index_by_id(primatives: &Query<&geometry::BoxGeometry>, id: &node_id::NodeId) -> u32 {
    primatives
        .iter()
        .position(|primative| primative.id == *id)
        .expect("should exist") as u32
}
