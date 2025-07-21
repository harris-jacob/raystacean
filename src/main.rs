mod camera;
mod geometry;
mod rendering;
mod maths;

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                ..default()
            }),
            rendering::RenderingPlugin,
            geometry::GeometryPlugin,
            camera::CameraPlugin,
        ))
        .run();
}
