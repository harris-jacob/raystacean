use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        gpu_readback::{Readback, ReadbackComplete},
        render_resource::{BufferUsages, Extent3d, TextureDimension, TextureFormat, TextureUsages},
        storage::ShaderStorageBuffer,
        view::RenderLayers,
    },
};

use crate::{controls, events, geometry, layers, rendering};

pub struct SelectionPlugin;

#[derive(Component)]
pub struct Selected;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, setup.after(rendering::setup));
        app.add_observer(box_selection);
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut materials: ResMut<Assets<rendering::SceneMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    gpu_boxes: Res<rendering::ShaderBufferHandle>,
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

    let selection_buffer = vec![0.0; 3];
    let mut selection_buffer = ShaderStorageBuffer::from(selection_buffer);
    selection_buffer.buffer_description.usage |= BufferUsages::COPY_SRC;
    let selection = buffers.add(selection_buffer);

    let material_handle = materials.add(rendering::SceneMaterial {
        aspect_ratio: Vec2::new(window.width(), window.height()),
        view_to_world: Mat4::default(),
        clip_to_view: Mat4::default(),
        is_color_picking: rendering::bool_to_gpu(true),
        boxes: gpu_boxes.inner().clone(),
        cursor_position: Vec2::default(),
        selection: selection.clone(),
    });

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
        RenderLayers::layer(layers::SHADER_LAYER),
    ));

    let mesh = meshes.add(Mesh::from(Plane3d::new(
        Vec3::Z,
        Vec2::new(window.width() * 0.5, window.height() * 0.5),
    )));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material_handle),
        RenderLayers::layer(layers::SHADER_LAYER),
    ));

    // TODO: can inline the system below
    commands.spawn(Readback::buffer(selection)).observe(
        |trigger: Trigger<ReadbackComplete>, mut ev: EventWriter<events::PixelColorUnderCursor>| {
            let data: Vec<f32> = trigger.event().to_shader_type();

            ev.write(events::PixelColorUnderCursor::new(Vec3::new(
                data[0], data[1], data[2],
            )));
        },
    );
}

fn box_selection(
    _trigger: Trigger<events::PlaneClicked>,
    control_mode: Res<controls::ControlMode>,
    selected: Query<Entity, With<Selected>>,
    boxes: Query<(Entity, &geometry::BoxGeometry)>,
    mut ev: EventReader<events::PixelColorUnderCursor>,
    mut commands: Commands,
) {
    if *control_mode != controls::ControlMode::Select {
        return;
    }

    // deselect existing
    for entity in selected.iter() {
        commands.entity(entity).remove::<Selected>();
    }

    if let Some(latest) = ev.read().last() {
        let id = geometry::GeometryId::from_color(latest.color());
        dbg!(id);

        if let Some(newly_selected) = boxes.iter().find(|(_, geometry)| geometry.id == id) {
            commands.entity(newly_selected.0).insert(Selected);
        }
    }
}
