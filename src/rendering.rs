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
use crate::{bvh, events};
use crate::{geometry, layers};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LitMaterial>::default())
            .add_plugins(MaterialPlugin::<SelectionMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    bvh_to_gpu,
                    boxes_to_gpu,
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
    mut lit_material: ResMut<Assets<LitMaterial>>,
    mut selection_material: ResMut<Assets<SelectionMaterial>>,
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
    let bvh = buffers.add(ShaderStorageBuffer::default());
    let prim_indices = buffers.add(ShaderStorageBuffer::default());

    let selection_buffer = vec![0.0; 3];
    let mut selection_buffer = ShaderStorageBuffer::from(selection_buffer);
    selection_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;

    let selection = buffers.add(selection_buffer);

    let lit_material_handle = lit_material.add(LitMaterial {
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        primatives: primatives.clone(),
        prim_indices: prim_indices.clone(),
        bvh: bvh.clone(),
    });

    let selection_material_handle = selection_material.add(SelectionMaterial {
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        primatives: primatives.clone(),
        bvh: bvh.clone(),
        prim_indices: prim_indices.clone(),
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
    commands.insert_resource(BvhBufferHandle(bvh));
    commands.insert_resource(PrimIndicesBuffer(prim_indices));

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
            MeshMaterial3d(selection_material_handle),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Mesh3d, With<RenderingPlane>>,
) {
    for e in resize_events.read() {
        let width = e.width;
        let height = e.height;

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

fn boxes_to_gpu(
    boxes: Query<&geometry::BoxGeometry>,
    buffer_handle: Res<PrimativesBufferHandle>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let buffer = buffer_handle.get_mut(&mut buffers);

    let gpu_data: Vec<GpuPrimative> = boxes
        .iter()
        // Sorted by ID to ensure stable operation ordering seen by the shader
        .sort_by::<&geometry::BoxGeometry>(|a, b| a.id.cmp(&b.id))
        .map(|b| GpuPrimative {
            position: b.position.into(),
            scale: b.scale.into(),
            color: b.color,
            blend: b.blend,
            rounding_radius: b.rounding_radius(),
            logical_color: b.id.to_color(),
            is_subtract: if b.is_subtract { 1 } else { 0 },
            ..default()
        })
        .collect();

    buffer.set_data(gpu_data);
}

fn bvh_to_gpu(
    bvh: Res<bvh::BVHTree>,
    bvh_handle: Res<BvhBufferHandle>,
    indices_handle: Res<PrimIndicesBuffer>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut gpu_data = Vec::new();
    let mut indices = Vec::new();

    flatten_bvh(&bvh.root, &mut gpu_data, &mut indices);

    let bvh_buffer = bvh_handle.get_mut(&mut buffers);
    bvh_buffer.set_data(gpu_data);

    let indices_buffer = indices_handle.get_mut(&mut buffers);
    indices_buffer.set_data(indices);
}

fn flatten_bvh(node: &bvh::BVHNode, flat: &mut Vec<GpuBvhNode>, indices: &mut Vec<u32>) -> i32 {
    let index = flat.len() as i32;

    match node {
        bvh::BVHNode::Leaf {
            bounds,
            prim_indices: local_indices,
        } => {
            let first_prim = indices.len() as u32;
            let prim_count = local_indices.len() as u32;

            for &idx in local_indices {
                indices.push(idx as u32);
            }

            flat.push(GpuBvhNode {
                bounds_min: [bounds.min.x, bounds.min.y, bounds.min.z],
                bounds_max: [bounds.max.x, bounds.max.y, bounds.max.z],
                left_index: -1,
                right_index: -1,
                first_prim,
                prim_count,
                ..default()
            });
        }
        bvh::BVHNode::Internal {
            bounds,
            left,
            right,
        } => {
            // Placeholder node
            flat.push(GpuBvhNode {
                bounds_min: [bounds.min.x, bounds.min.y, bounds.min.z],
                bounds_max: [bounds.max.x, bounds.max.y, bounds.max.z],
                left_index: -1,
                right_index: -1,
                first_prim: 0,
                prim_count: 0,
                ..default()
            });

            let left_index = flatten_bvh(left, flat, indices);
            let right_index = flatten_bvh(right, flat, indices);

            // Update the parent node now that we know child indices
            flat[index as usize].left_index = left_index;
            flat[index as usize].right_index = right_index;
        }
    }

    index
}

fn cursor_position(windows: Query<&Window>, mut materials: ResMut<Assets<SelectionMaterial>>) {
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
#[derive(Clone, Copy, Debug, ShaderType, Default)]
struct GpuBvhNode {
    bounds_min: [f32; 3],
    _pad0: f32, // pad to 16 bytes
    bounds_max: [f32; 3],
    _pad1: f32, // pad to 16 bytes
    left_index: i32,
    right_index: i32,
    first_prim: u32,
    prim_count: u32,
}

#[repr(C)]
#[derive(Clone, ShaderType, Default, Debug)]
pub struct GpuPrimative {
    pub position: [f32; 3],
    pub is_subtract: u32,
    pub scale: [f32; 3],
    pub blend: f32,
    pub color: [f32; 3],
    pub rounding_radius: f32,
    pub logical_color: [f32; 3],
    _pad1: f32,
}

/// Material linked to shader that displays only primative shapes, rendering
/// each shape according to a color representation of its ID, used for color
/// picking selection.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SelectionMaterial {
    #[uniform(0)]
    pub view_to_world: Mat4,
    #[uniform(1)]
    pub clip_to_view: Mat4,
    #[uniform(2)]
    pub cursor_position: Vec2,
    #[storage(3, read_only)]
    pub primatives: Handle<ShaderStorageBuffer>,
    #[storage(4, read_only)]
    pub bvh: Handle<ShaderStorageBuffer>,
    #[storage(5, read_only)]
    pub prim_indices: Handle<ShaderStorageBuffer>,
    #[storage(6)]
    pub selection: Handle<ShaderStorageBuffer>,
}

/// Material linked to shader that displays the scene with full lighting and
/// takes into account CSG operations.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LitMaterial {
    #[uniform(0)]
    pub view_to_world: Mat4,
    #[uniform(1)]
    pub clip_to_view: Mat4,
    #[storage(2, read_only)]
    pub primatives: Handle<ShaderStorageBuffer>,
    #[storage(3, read_only)]
    pub bvh: Handle<ShaderStorageBuffer>,
    #[storage(4, read_only)]
    pub prim_indices: Handle<ShaderStorageBuffer>,
}

#[derive(Resource)]
pub struct PrimativesBufferHandle(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct BvhBufferHandle(Handle<ShaderStorageBuffer>);

#[derive(Resource)]
pub struct PrimIndicesBuffer(Handle<ShaderStorageBuffer>);

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

impl BvhBufferHandle {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }
}

impl PrimIndicesBuffer {
    pub fn get_mut<'a>(
        &self,
        assets: &'a mut Assets<ShaderStorageBuffer>,
    ) -> &'a mut ShaderStorageBuffer {
        assets
            .get_mut(&self.0)
            .expect("ShaderStorageBuffer should exist")
    }
}

impl Material for SelectionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/selection_shader.wgsl".into()
    }
}

impl Material for LitMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lit_shader.wgsl".into()
    }
}
