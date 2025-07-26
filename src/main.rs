mod camera;
mod controls;
mod events;
mod geometry;
mod rendering;
mod selection;

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_event::<events::PixelColorUnderCursor>()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                ..default()
            }),
            rendering::RenderingPlugin,
            geometry::GeometryPlugin,
            camera::CameraPlugin,
            selection::SelectionPlugin,
            controls::ControlContextPlugin,
        ))
        .run();
}
