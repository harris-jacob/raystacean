mod camera;
mod controls;
mod events;
mod geometry;
mod gizmos;
mod layers;
mod rendering;
mod selection;
mod ui;

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_event::<events::PixelColorUnderCursor>()
        .add_event::<events::PlaneClicked>()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::DEBUG,
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .add_plugins((
            camera::CameraPlugin,
            controls::ControlContextPlugin,
            geometry::GeometryPlugin,
            gizmos::GizmosPlugin,
            rendering::RenderingPlugin,
            selection::SelectionPlugin,
            ui::UiPlugin,
        ))
        .run();
}
