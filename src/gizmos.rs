use crate::{events, geometry, layers, selection};
use bevy::color::palettes::css::{BLUE, GREEN, RED};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct GizmosPlugin;

#[derive(Component, Debug)]
pub struct Origin;

#[derive(Component, Debug)]
pub struct ScalingGizmo(Axis);

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (draw_coordinate_system, draw_scaling_cubes));
    }
}

// Draw a coordinate system for the selected box
fn draw_coordinate_system(
    selected: Query<&geometry::BoxGeometry, With<selection::Selected>>,
    mut origin: Query<(&mut Transform, &mut Visibility), With<Origin>>,
) {
    let (mut transform, mut visibility) = origin.single_mut().expect("single");

    if let Ok(selected) = selected.single() {
        *visibility = Visibility::Visible;
        *transform = Transform::from_translation(selected.position);
    } else {
        *visibility = Visibility::Hidden;
    }
}

// Draw a coordinate system for the selected box
fn draw_scaling_cubes(
    selected: Query<&geometry::BoxGeometry, With<selection::Selected>>,
    scaling_cube: Query<(&mut Transform, &mut Visibility, &ScalingGizmo)>,
) {
    for (mut transform, mut visibility, ScalingGizmo(axis)) in scaling_cube {
        if let Ok(selected) = selected.single() {
            *visibility = Visibility::Visible;
            *transform = match axis {
                Axis::X => Transform::from_translation(
                    selected.position + vec3(selected.scale.x, 0.0, 0.0),
                ),
                Axis::Y => Transform::from_translation(
                    selected.position + vec3(0.0, selected.scale.y, 0.0),
                ),
                Axis::Z => Transform::from_translation(
                    selected.position + vec3(0.0, 0.0, selected.scale.z),
                ),
            };
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let line_mesh = meshes.add(Mesh::from(Cylinder {
        radius: 0.05,
        half_height: 1.0,
    }));

    let cone_mesh = meshes.add(Mesh::from(Cone {
        radius: 0.2,
        height: 0.5,
    }));

    let cube_mesh = meshes.add(Mesh::from(Cuboid {
        half_size: vec3(0.15, 0.15, 0.15),
    }));

    let x_material = materials.add(StandardMaterial {
        base_color: color_for_axis(Axis::X),
        unlit: true,
        ..default()
    });

    let y_material = materials.add(StandardMaterial {
        base_color: color_for_axis(Axis::Y),
        unlit: true,
        ..default()
    });

    let z_material = materials.add(StandardMaterial {
        base_color: color_for_axis(Axis::Z),
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Transform::default(),
            Visibility::Hidden,
            Origin,
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .with_children(|parent| {
            draw_origin_axis(
                parent,
                x_material.clone(),
                line_mesh.clone(),
                cone_mesh.clone(),
                Axis::X,
            );
            draw_origin_axis(
                parent,
                y_material.clone(),
                line_mesh.clone(),
                cone_mesh.clone(),
                Axis::Y,
            );
            draw_origin_axis(
                parent,
                z_material.clone(),
                line_mesh.clone(),
                cone_mesh.clone(),
                Axis::Z,
            );
        });

    commands
        .spawn((
            ScalingGizmo(Axis::X),
            Visibility::Hidden,
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(x_material.clone()),
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .observe(make_drag_scaling_gizmo(Axis::X));

    commands
        .spawn((
            ScalingGizmo(Axis::Y),
            Visibility::Hidden,
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(y_material.clone()),
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .observe(make_drag_scaling_gizmo(Axis::Y));

    commands
        .spawn((
            ScalingGizmo(Axis::Z),
            Visibility::Hidden,
            Mesh3d(cube_mesh),
            MeshMaterial3d(z_material.clone()),
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .observe(make_drag_scaling_gizmo(Axis::Z));
}

fn make_drag_scaling_gizmo(
    axis: Axis,
) -> impl Fn(Trigger<Pointer<Drag>>, EventWriter<events::ScalingGizmoDragged>) {
    move |drag: Trigger<Pointer<Drag>>,
          mut event_writer: EventWriter<events::ScalingGizmoDragged>| {
        event_writer.write(events::ScalingGizmoDragged {
            delta: drag.delta,
            axis: axis.to_vec(),
        });
    }
}

fn make_drag_origin(
    axis: Axis,
) -> impl Fn(Trigger<Pointer<Drag>>, EventWriter<events::OriginDragged>) {
    move |drag: Trigger<Pointer<Drag>>, mut event_writer: EventWriter<events::OriginDragged>| {
        event_writer.write(events::OriginDragged {
            delta: drag.delta,
            axis: axis.to_vec(),
        });
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn to_vec(self) -> Vec3 {
        match self {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
        }
    }
}

fn draw_origin_axis(
    commands: &mut RelatedSpawnerCommands<ChildOf>,
    material_handle: Handle<StandardMaterial>,
    line_mesh: Handle<Mesh>,
    cone_mesh: Handle<Mesh>,
    axis: Axis,
) {
    let base_transform = transform_for_axis(axis);

    commands.spawn((
        Mesh3d(line_mesh),
        base_transform,
        MeshMaterial3d(material_handle.clone()),
        RenderLayers::layer(layers::GIZMOS_LAYER),
    ));

    commands
        .spawn((
            Mesh3d(cone_mesh),
            base_transform.with_translation(base_transform.translation * 2.0),
            MeshMaterial3d(material_handle),
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .observe(make_drag_origin(axis));
}

fn transform_for_axis(axis: Axis) -> Transform {
    match axis {
        Axis::X => Transform::from_translation(Vec3::X)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        Axis::Y => Transform::from_translation(Vec3::Y),
        Axis::Z => Transform::from_translation(Vec3::Z)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    }
}

fn color_for_axis(axis: Axis) -> Color {
    match axis {
        Axis::X => RED.into(),
        Axis::Y => GREEN.into(),
        Axis::Z => BLUE.into(),
    }
}
