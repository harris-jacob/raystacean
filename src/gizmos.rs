use crate::{camera, geometry, layers};
use bevy::{color::palettes::css::RED, prelude::*, render::view::RenderLayers};

pub struct GizmosPlugin;

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update_gizmo_camera, draw_coordinate_system));
    }
}

#[derive(Debug, Component)]
struct GizmoCamera;

fn setup(mut commands: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.render_layers = RenderLayers::layer(layers::GIZMOS_LAYER);

    commands.spawn((
        GizmoCamera,
        Camera3d::default(),
        Camera {
            order: layers::GIZMOS_CAMERA,
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_2,
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }),
        RenderLayers::layer(layers::GIZMOS_LAYER),
    ));
}

/// Keep the gizmo camera in sync with the 'main' camera
fn update_gizmo_camera(
    camera: ResMut<camera::CameraControls>,
    mut query: Query<(&GizmoCamera, &mut Transform)>,
) {
    let (_, mut transform) = query.single_mut().expect("one should exist");

    let rotation = Quat::from_euler(EulerRot::YXZ, camera.azimuth, camera.elevation, 0.0);
    let offset = rotation * Vec3::new(0.0, 0.0, camera.distance);
    let position = camera.target + offset;

    *transform = Transform {
        translation: position,
        rotation,
        scale: Vec3::ONE,
    };

    dbg!(transform);
}

// Draw a coordinate system for every box
fn draw_coordinate_system(
    boxes: Query<&geometry::BoxGeometry>,
    mut gizmos: Gizmos,
    camera: ResMut<camera::CameraControls>,
) {
    gizmos.axes(Transform::default(), 1.0);

    for b in boxes {
        gizmos.axes(Transform::from_translation(-b.position), 1.0);
    }
}
