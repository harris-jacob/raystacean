mod camera;
mod rendering;
mod geometry;

use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new().add_plugins((DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        ..default()
    }),));
}

fn id_to_color(id: u32) -> Vec3 {
    let r = (id & 0xFF) / 255;
    let g = ((id >> 8) & 0xFF) / 255;
    let b = ((id >> 16) & 0xFF) / 255;

    Vec3::new(r as f32, g as f32, b as f32)
}
