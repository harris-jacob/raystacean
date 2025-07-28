mod camera;
mod controls;
mod events;
mod geometry;
mod rendering;
mod selection;
mod ui;

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
            camera::CameraPlugin,
            controls::ControlContextPlugin,
            geometry::GeometryPlugin,
            rendering::RenderingPlugin,
            selection::SelectionPlugin,
            ui::UiPlugin,
        ))
        .run();
}
