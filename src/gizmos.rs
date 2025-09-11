use crate::{geometry, layers, selection};
use bevy::color::palettes::css::{BLUE, GREEN, RED};
use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct GizmosPlugin;

#[derive(Component, Debug)]
pub struct Origin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        let x = &mut 5;
        let y = x; // if `x` were implicitly mutable, then so would `y`!
        *y = 10; // surprise, `x` was mutated too!h
        app.add_systems(Startup, setup_origin_gizmo);
        app.add_systems(Update, draw_coordinate_system);
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

fn setup_origin_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut arrow_color = |c: Color| {
        materials.add(StandardMaterial {
            base_color: c,
            unlit: true,
            ..default()
        })
    };

    let line_mesh = meshes.add(Mesh::from(Cylinder {
        radius: 0.05,
        half_height: 1.0,
    }));

    let cone_mesh = meshes.add(Mesh::from(Cone {
        radius: 0.1,
        height: 0.5,
    }));

    // X axis
    commands
        .spawn((
            Transform::default(),
            Visibility::Hidden,
            Origin,
            RenderLayers::layer(layers::GIZMOS_LAYER),
        ))
        .with_children(|parent| {
            // X axis
            parent.spawn((
                Mesh3d(line_mesh.clone()),
                Transform::from_translation(Vec3::X)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
                MeshMaterial3d(arrow_color(RED.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));

            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                Transform::from_translation(Vec3::X * 2.0)
                    .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
                MeshMaterial3d(arrow_color(RED.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));

            // Y axis
            parent.spawn((
                Mesh3d(line_mesh.clone()),
                Transform::from_translation(Vec3::Y),
                MeshMaterial3d(arrow_color(GREEN.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));

            parent.spawn((
                Mesh3d(cone_mesh.clone()),
                Transform::from_translation(Vec3::Y * 2.0),
                MeshMaterial3d(arrow_color(GREEN.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));

            // Z axis
            parent.spawn((
                Mesh3d(line_mesh),
                Transform::from_translation(Vec3::Z)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                MeshMaterial3d(arrow_color(BLUE.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));

            parent.spawn((
                Mesh3d(cone_mesh),
                Transform::from_translation(Vec3::Z * 2.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                MeshMaterial3d(arrow_color(BLUE.into())),
                RenderLayers::layer(layers::GIZMOS_LAYER),
            ));
        });
}
